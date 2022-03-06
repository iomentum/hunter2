mod aux_type;

use aux_type::AuxTypeGenerator;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Comma, Paren},
    Block, DeriveInput, Expr, ExprCall, ExprPath, ExprReference, FnArg, Generics, Ident,
    ImplItemMethod, Item, ItemImpl, PatIdent, PatType, Path, PathArguments, PathSegment, Receiver,
    ReturnType, Signature, Stmt, Token, Type, TypePath, TypeReference, VisPublic, Visibility,
};

#[proc_macro_derive(Hunter2, attributes(hidden))]
pub fn hunter_2(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);
    MacroCall::new(input).expand().to_token_stream().into()
}

struct MacroCall {
    inner_data: DeriveInput,
}

impl MacroCall {
    fn new(data: DeriveInput) -> MacroCall {
        MacroCall { inner_data: data }
    }

    fn expand(&self) -> ItemImpl {
        let generics = self.inner_data.generics.clone();
        let trait_ = Some(self.trait_());
        let self_ty = Box::new(self.self_ty().into());
        let brace_token = Brace::default();
        let items = vec![self.fmt_method().into()];

        ItemImpl {
            attrs: Vec::new(),
            defaultness: None,
            unsafety: None,
            impl_token: mk_token(Token![impl]),
            generics,
            trait_,
            self_ty,
            brace_token,
            items,
        }
    }

    fn trait_(&self) -> (Option<Token![!]>, Path, Token![for]) {
        (
            None,
            path_from_strs(LeadingColon::Yes, ["core", "fmt", "Debug"]),
            mk_token(Token![for]),
        )
    }

    fn self_ty(&self) -> TypePath {
        TypePath {
            qself: None,
            path: path_from_ident(self.inner_data.ident.clone()),
        }
    }

    fn fmt_method(&self) -> ImplItemMethod {
        let sig = self.fmt_method_sig();
        let block = self.fmt_method_block();

        ImplItemMethod {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig,
            block,
        }
    }

    fn fmt_method_sig(&self) -> Signature {
        let generics = self.fmt_method_generics();
        let inputs = self.fmt_method_inputs();
        let output = self.fmt_method_output();

        Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: mk_token(Token!(fn)),
            ident: Ident::new("fmt", Span::call_site()),
            generics,
            paren_token: paren(),
            inputs,
            variadic: None,
            output,
        }
    }

    fn fmt_method_generics(&self) -> Generics {
        Generics::default()
    }

    fn fmt_method_inputs(&self) -> Punctuated<FnArg, Comma> {
        let self_: FnArg = and_self_fn_arg().into();
        let formatter = self.formatter_fn_arg().into();

        [self_, formatter].into_iter().collect()
    }

    fn fmt_method_output(&self) -> ReturnType {
        let ty = Type::Path(TypePath {
            qself: None,
            path: path_from_strs(LeadingColon::Yes, ["core", "fmt", "Result"]),
        });

        ReturnType::Type(mk_token(Token![->]), Box::new(ty))
    }

    fn formatter_fn_arg(&self) -> PatType {
        let pat = PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: Ident::new("f", Span::call_site()),
            subpat: None,
        }
        .into();

        let ty = TypeReference {
            and_token: mk_token(Token![&]),
            lifetime: None,
            mutability: Some(mk_token(Token![mut])),
            elem: Box::new(self.formatter_path().into()),
        }
        .into();

        PatType {
            attrs: Vec::new(),
            pat: Box::new(pat),
            colon_token: mk_token(Token![:]),
            ty: Box::new(ty),
        }
    }

    fn formatter_path(&self) -> TypePath {
        TypePath {
            qself: None,
            path: path_from_strs(LeadingColon::Yes, ["core", "fmt", "Formatter"]),
        }
    }

    fn fmt_method_block(&self) -> Block {
        let aux_ty_decl: Item = self.aux_type_declaration();
        let fmt_call = self.fmt_call();

        let stmts = vec![Stmt::Item(aux_ty_decl), Stmt::Expr(fmt_call)];

        Block {
            brace_token: Brace::default(),
            stmts,
        }
    }

    fn aux_type_declaration(&self) -> Item {
        AuxTypeGenerator::from(&self.inner_data).expand_declaration()
    }

    fn fmt_call(&self) -> Expr {
        let func = Box::new(Self::fmt_method_path().into());
        let args = self.fmt_call_args();

        ExprCall {
            attrs: Vec::new(),
            func,
            paren_token: Paren::default(),
            args,
        }
        .into()
    }

    fn fmt_method_path() -> ExprPath {
        ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path_from_strs(LeadingColon::Yes, ["core", "fmt", "Debug", "fmt"]),
        }
    }

    fn fmt_call_args(&self) -> Punctuated<Expr, Comma> {
        let aux_type_creation_ref = Expr::Reference(ExprReference {
            attrs: Vec::new(),
            and_token: mk_token(Token![&]),
            raw: Default::default(),
            mutability: None,
            expr: Box::new(self.aux_type_creation()),
        });

        let formatter = Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path_from_ident(Ident::new("f", Span::call_site())),
        });

        [aux_type_creation_ref, formatter].into_iter().collect()
    }

    fn aux_type_creation(&self) -> Expr {
        AuxTypeGenerator::from(&self.inner_data).expand_creation()
    }
}

fn mk_token<T>(maker: fn(Span) -> T) -> T {
    maker(Span::call_site())
}

fn path_from_strs<'a>(
    leading_colon: LeadingColon,
    segments: impl IntoIterator<Item = &'a str>,
) -> Path {
    let idents = segments
        .into_iter()
        .map(|name| Ident::new(name, Span::call_site()));

    path_from_idents(leading_colon, idents)
}

fn path_from_ident(ident: Ident) -> Path {
    path_from_idents(LeadingColon::No, [ident])
}

fn path_from_idents(
    leading_colon: LeadingColon,
    segments: impl IntoIterator<Item = Ident>,
) -> Path {
    let segments = segments
        .into_iter()
        .map(|ident| PathSegment {
            ident,
            arguments: PathArguments::None,
        })
        .collect();

    let leading_colon = match leading_colon {
        LeadingColon::Yes => Some(mk_token(Token![::])),
        LeadingColon::No => None,
    };

    Path {
        leading_colon,
        segments,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LeadingColon {
    Yes,
    No,
}

fn paren() -> Paren {
    Paren {
        span: Span::call_site(),
    }
}

fn and_self_fn_arg() -> Receiver {
    Receiver {
        attrs: Vec::new(),
        reference: Some((mk_token(Token![&]), None)),
        mutability: None,
        self_token: mk_token(Token![self]),
    }
}

fn pub_kw() -> Visibility {
    Visibility::Public(VisPublic {
        pub_token: mk_token(Token![pub]),
    })
}
