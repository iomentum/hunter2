use std::{iter, mem};

use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Comma},
    AngleBracketedGenericArguments, Arm, AttrStyle, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Expr, ExprCall, ExprMatch, ExprPath, ExprStruct, Field, FieldPat, FieldValue,
    Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Generics, Ident, Index, Item, ItemEnum,
    ItemStruct, LifetimeDef, Member, Pat, PatIdent, PatStruct, Path, PathArguments, PathSegment,
    Token, Type, TypePath, TypeReference, Variant,
};

use crate::{mk_token, path_from_ident, path_from_strs, pub_kw, LeadingColon};

pub(crate) struct AuxTypeGenerator<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
    data: &'a Data,
}

impl<'a> AuxTypeGenerator<'a> {
    pub(crate) fn from(input: &'a DeriveInput) -> AuxTypeGenerator<'a> {
        let DeriveInput {
            ident,
            generics,
            data,
            ..
        } = input;

        AuxTypeGenerator {
            ident,
            generics,
            data,
        }
    }

    pub(crate) fn expand_declaration(&self) -> Item {
        match self.data {
            Data::Struct(s) => AuxStructGenerator::new(self.ident, self.generics, s)
                .expand_declaration()
                .into(),
            Data::Enum(e) => AuxEnumGenerator::new(self.ident, self.generics, e)
                .expand_declaration()
                .into(),
            Data::Union(_) => todo!(),
        }
    }

    pub(crate) fn expand_creation(&self) -> Expr {
        match self.data {
            Data::Struct(s) => AuxStructGenerator::new(self.ident, self.generics, s)
                .expand_creation()
                .into(),
            Data::Enum(e) => AuxEnumGenerator::new(self.ident, self.generics, e)
                .expand_creation()
                .into(),
            Data::Union(_) => todo!(),
        }
    }
}

pub(crate) struct AuxStructGenerator<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
    data: &'a DataStruct,
}

impl<'a> AuxStructGenerator<'a> {
    fn new(
        ident: &'a Ident,
        generics: &'a Generics,
        data: &'a DataStruct,
    ) -> AuxStructGenerator<'a> {
        AuxStructGenerator {
            ident,
            generics,
            data,
        }
    }

    fn expand_declaration(self) -> ItemStruct {
        let attrs = vec![derive_debug_attr()];
        let fields = self.fields();
        let semi_token = Some(mk_token(Token![;]));
        let generics = self.generics();

        ItemStruct {
            attrs,
            vis: pub_kw(),
            struct_token: mk_token(Token![struct]),
            ident: self.ident.clone(),
            generics,
            fields,
            semi_token,
        }
    }

    fn fields(&self) -> Fields {
        SumType::from(&self.data.fields).expand()
    }

    fn generics(&self) -> Generics {
        let mut generics = self.generics.clone();
        generics.params.push(borrow_lifetime().into());
        generics
    }

    fn expand_creation(self) -> ExprMatch {
        let arm = self.arm();
        let arms = vec![arm];
        ExprMatch {
            attrs: Vec::new(),
            match_token: mk_token(Token![match]),
            expr: Box::new(self_()),
            brace_token: Brace::default(),
            arms,
        }
    }

    fn arm(&self) -> Arm {
        SumType::from(&self.data.fields).arm(self.ident.clone())
    }
}

pub(crate) struct AuxEnumGenerator<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
    data: &'a DataEnum,
}

impl<'a> AuxEnumGenerator<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics, data: &'a DataEnum) -> AuxEnumGenerator<'a> {
        AuxEnumGenerator {
            ident,
            generics,
            data,
        }
    }

    fn expand_declaration(&self) -> ItemEnum {
        let attr = Self::derive_debug();
        let attrs = vec![attr];
        let variants = self.variants();

        ItemEnum {
            attrs,
            vis: pub_kw(),
            enum_token: mk_token(Token![enum]),
            ident: self.ident.clone(),
            generics: self.generics.clone(),
            brace_token: Brace::default(),
            variants,
        }
    }

    fn derive_debug() -> Attribute {
        let tokens = quote! { (Debug) };

        Attribute {
            pound_token: mk_token(Token![#]),
            style: AttrStyle::Outer,
            bracket_token: Bracket::default(),
            path: path_from_ident(Ident::new("derive", Span::call_site())),
            tokens,
        }
    }

    fn variants(&self) -> Punctuated<Variant, Comma> {
        self.data.variants.iter().map(Self::mk_variant).collect()
    }

    fn mk_variant(v: &Variant) -> Variant {
        let Variant {
            attrs,
            ident,
            fields,
            discriminant,
        } = v;

        let attrs = attrs.clone();
        let ident = ident.clone();
        let discriminant = discriminant.clone();

        let fields = SumType::from(fields).expand();

        Variant {
            attrs,
            ident,
            fields,
            discriminant,
        }
    }

    fn expand_creation(self) -> ExprMatch {
        todo!()
    }
}

struct SumType<'a> {
    fields: &'a Fields,
}

impl<'a> SumType<'a> {
    fn from(fields: &'a Fields) -> SumType<'a> {
        SumType { fields }
    }

    fn expand(&self) -> Fields {
        let mut fields = self.fields.clone();

        match fields {
            Fields::Named(FieldsNamed { ref mut named, .. }) => Self::alter_fields(named),
            Fields::Unnamed(FieldsUnnamed {
                ref mut unnamed, ..
            }) => Self::alter_fields(unnamed),
            Fields::Unit => {}
        }

        fields
    }

    fn alter_fields(fields: &mut Punctuated<Field, Comma>) {
        fields.iter_mut().for_each(Self::alter_field)
    }

    fn alter_field(field: &mut Field) {
        if Self::has_hidden_attr(field) {
            Self::wrap_type_in_hidden(&mut field.ty);
            Self::remove_attrs(field);
        } else {
            Self::wrap_type_in_ref(&mut field.ty);
        }
    }

    fn has_hidden_attr(field: &Field) -> bool {
        let Field { attrs, .. } = field;

        attrs.iter().any(Self::is_hidden_attr)
    }

    fn is_hidden_attr(attr: &Attribute) -> bool {
        let meta = attr.parse_meta().unwrap();

        let path = match meta {
            syn::Meta::Path(path) => path,
            _ => return false,
        };

        let ident = match path.get_ident() {
            Some(ident) => ident,
            None => return false,
        };

        ident == "hidden"
    }

    fn wrap_type_in_hidden(ty: &mut Type) {
        let inner_ty = mem::replace(ty, Type::Verbatim(quote! {}));
        let path = Self::hidden_type_path(inner_ty);

        let wrapped = TypePath { qself: None, path }.into();

        *ty = wrapped;
    }

    fn hidden_type_path(generic_ty: Type) -> Path {
        let parent_mod = PathSegment {
            ident: Ident::new("hunter2", Span::call_site()),
            arguments: PathArguments::None,
        };
        let final_segment = PathSegment {
            ident: Ident::new("Hidden", Span::call_site()),
            arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: mk_token(Token![<]),
                args: [
                    GenericArgument::Lifetime(borrow_lifetime().lifetime),
                    GenericArgument::Type(generic_ty),
                ]
                .into_iter()
                .collect(),
                gt_token: mk_token(Token![>]),
            }),
        };

        Path {
            leading_colon: Some(mk_token(Token![::])),
            segments: [parent_mod, final_segment].into_iter().collect(),
        }
    }

    fn remove_attrs(field: &mut Field) {
        field.attrs.clear()
    }

    fn wrap_type_in_ref(ty: &mut Type) {
        let tmp = Type::Verbatim(quote! {});
        let initial_ty = mem::replace(ty, tmp);
        let ref_type = Type::Reference(TypeReference {
            and_token: mk_token(Token![&]),
            lifetime: Some(borrow_lifetime().lifetime),
            mutability: None,
            elem: Box::new(initial_ty),
        });
        *ty = ref_type;
    }

    fn arm(&self, type_name: Ident) -> Arm {
        let pat = self.pat();
        let body = Box::new(self.body(path_from_ident(type_name)));
        Arm {
            attrs: Vec::new(),
            pat,
            guard: None,
            fat_arrow_token: mk_token(Token![=>]),
            body,
            comma: Some(mk_token(Token![,])),
        }
    }

    fn pat(&self) -> Pat {
        let path = path_from_ident(Ident::new("Self", Span::call_site()));
        let fields = self.pat_fields();

        Pat::Struct(PatStruct {
            attrs: Vec::new(),
            path,
            brace_token: Brace::default(),
            fields,
            dot2_token: None,
        })
    }

    fn pat_fields(&self) -> Punctuated<FieldPat, Comma> {
        self.fields
            .iter()
            .enumerate()
            .map(|(idx, field)| (&field.ident, Self::mk_field_name(idx)))
            .map(Self::mk_pat_field)
            .collect()
    }

    fn mk_field_name(idx: usize) -> Ident {
        format_ident!("field_{}", idx)
    }

    fn mk_pat_field((field_ident, binding_ident): (&Option<Ident>, Ident)) -> FieldPat {
        let member = match field_ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index {
                index: 0,
                span: Span::call_site(),
            }),
        };

        let pat = Pat::Ident(PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: binding_ident,
            subpat: None,
        });

        FieldPat {
            attrs: Vec::new(),
            member,
            colon_token: Some(mk_token(Token![:])),
            pat: Box::new(pat),
        }
    }

    fn body(&self, path: Path) -> Expr {
        let fields = self.body_fields();

        Expr::Struct(ExprStruct {
            attrs: Vec::new(),
            path,
            brace_token: Brace::default(),
            fields,
            dot2_token: None,
            rest: None,
        })
    }

    fn body_fields(&self) -> Punctuated<FieldValue, Comma> {
        self.fields
            .iter()
            .enumerate()
            .map(|(idx, field)| (&field.ident, Self::mk_field_name(idx)))
            .map(Self::mk_field_value)
            .collect()
    }

    fn mk_field_value((pat_ident, binding_ident): (&Option<Ident>, Ident)) -> FieldValue {
        let member = match pat_ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(Index {
                index: 0,
                span: Span::call_site(),
            }),
        };

        let expr = Self::mk_field_value_expr(binding_ident).into();

        FieldValue {
            attrs: Vec::new(),
            member,
            colon_token: Some(mk_token(Token![:])),
            expr,
        }
    }

    fn mk_field_value_expr(binding_ident: Ident) -> ExprCall {
        let func = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: into_method_path(),
        }
        .into();

        let arg = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: path_from_ident(binding_ident),
        };

        let args = iter::once(arg).map(Expr::from).collect();

        ExprCall {
            attrs: Vec::new(),
            func: Box::new(func),
            paren_token: Default::default(),
            args,
        }
    }
}

fn self_() -> Expr {
    Expr::Path(ExprPath {
        attrs: Vec::new(),
        qself: None,
        path: path_from_ident(Ident::new("self", Span::call_site())),
    })
}

fn borrow_lifetime() -> LifetimeDef {
    LifetimeDef {
        attrs: Vec::new(),
        lifetime: syn::Lifetime {
            apostrophe: Span::call_site(),
            ident: Ident::new("__hunter2", Span::call_site()),
        },
        colon_token: None,
        bounds: Punctuated::new(),
    }
}

fn into_method_path() -> Path {
    path_from_strs(LeadingColon::Yes, ["core", "convert", "Into", "into"])
}

fn derive_debug_attr() -> Attribute {
    Attribute {
        pound_token: mk_token(Token![#]),
        style: AttrStyle::Outer,
        bracket_token: Bracket::default(),
        path: path_from_ident(Ident::new("derive", Span::call_site())),
        tokens: quote! { (Debug) },
    }
}
