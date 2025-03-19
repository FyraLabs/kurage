use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::{parse::Parse, punctuated::Punctuated, Token};

struct GenerateGeneratorSyn {
    macroname: syn::Ident,
    component: Option<proc_macro2::TokenTree>,
    structblk: Option<proc_macro2::TokenStream>,
    initblk: Option<proc_macro2::TokenStream>,
    updateblk: Option<proc_macro2::TokenStream>,
    updateout: Option<Punctuated<proc_macro2::TokenStream, Token![,]>>,
    view_first: Option<proc_macro2::TokenTree>,
    views: Option<proc_macro2::TokenStream>,
}

impl syn::parse::Parse for GenerateGeneratorSyn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let macroname = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let mut out = Self {
            macroname,
            component: None,
            structblk: None,
            initblk: None,
            updateblk: None,
            updateout: None,
            view_first: None,
            views: None,
        };
        let mut next = loop {
            match input.parse::<TokenTree>()? {
                TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Bracket => {
                    // [< paste >]
                    if out.component.is_some() {
                        return Err(syn::Error::new(
                            g.span(),
                            "kurage: you may specify [< paste >] once only.",
                        ));
                    }
                    out.component = Some(g.into());
                    continue;
                }
                TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Brace => {
                    // { structblk }
                    out.structblk = Some(g.stream());
                    input.parse::<syn::Token![:]>()?;
                    break input.parse()?;
                }
                TokenTree::Group(g) => {
                    return Err(syn::Error::new(g.span(), "kurage: unexpected token. Pass in [<$name Page>] for a custom naming scheme (look at the paste crate), or provide a { field1: Type, field2: Type2 } block."));
                }
                TokenTree::Punct(p) if p.as_char() == ':' && out.component.is_some() => {
                    break input.parse()?;
                }
                TokenTree::Punct(p) if p.as_char() == ':' => {
                    return Err(syn::Error::new(
                        p.span(),
                        "kurage: missing [< paste Component >] before `:`",
                    ))
                }
                x => break x,
            }
        };
        match &next {
            proc_macro2::TokenTree::Ident(i) if i == "init" => {
                input.parse::<syn::Token![:]>()?;
                let t = input.parse::<Group>()?;
                if t.delimiter() != Delimiter::Brace {
                    return Err(syn::Error::new(t.span(), "kurage: expected { ... }"));
                }
                out.initblk = Some(t.stream());
                next = input.parse()?;
            }
            _ => (),
        }
        match &next {
            proc_macro2::TokenTree::Ident(i) if i == "update" => {
                struct Idk(Punctuated<TokenStream, Token![,]>);
                impl Parse for Idk {
                    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                        input
                            .parse_terminated(TokenStream::parse, Token![,])
                            .map(Self)
                    }
                }
                input.parse::<syn::Token![:]>()?;
                let t = input.parse::<Group>()?;
                if t.delimiter() != Delimiter::Brace {
                    return Err(syn::Error::new(t.span(), "kurage: expected { ... }"));
                }
                out.updateblk = Some(t.stream());
                input.parse::<syn::Token![=>]>()?;
                let t = input.parse::<Group>()?;
                if t.delimiter() != Delimiter::Brace {
                    return Err(syn::Error::new(t.span(), "kurage: expected { ... }"));
                }
                out.updateout = Some(syn::parse2::<Idk>(t.stream())?.0);
                next = input.parse()?;
            }
            _ => (),
        }
        out.view_first = Some(next);
        out.views = Some(input.parse()?);
        Ok(out)
    }
}

fn recurse_replace<'a, T: FnMut(proc_macro2::Ident) -> Box<dyn Iterator<Item = TokenTree>>>(
    ts: TokenStream,
    ident: &'a str,
    f: &'a mut T,
) -> impl Iterator<Item = TokenTree> + use<'a, T> {
    ts.into_iter().flat_map(move |tt| match tt {
        TokenTree::Ident(i) if i == ident => f(i),
        TokenTree::Group(group) => {
            Box::new(std::iter::once(TokenTree::Group(proc_macro2::Group::new(
                group.delimiter(),
                recurse_replace(group.stream(), ident, f).collect(),
            ))))
        }
        other => Box::new(std::iter::once(other)),
    })
}

/// Generate a **`macro_rules!`** that has a similar syntax to
/// [`kurage::generate_component!`], except that the new macro generates components
/// wrapped in a custom tree of widgets.
///
/// If you don't understand what this means, just look at the example.
///
/// The macro accepts an optional argument for the format of the names of the new components
/// specified in a format accepted by [`paste::paste!`] using the `$name` metavariable.
///
/// # Customization
///
/// For more information, consult [`kurage::kurage_gen_macros!`].
///
/// - `kurage_page_pre!`: code pasted before declaring the new [`relm4::SimpleComponent`].
///   Default:
///   ```rs
///   use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
///   ```
///
/// # Examples
///
/// Example extracted from [Taidan]: <https://github.com/Ultramarine-Linux/taidan/blob/2122fb2200c9b828d64be30ad734237a939da07e/src/macros.rs#L23-L39>
///
/// ```
/// # pub use kurage_proc_macros::generate_generator;
/// use kurage::relm4::{self, prelude::*};
/// use kurage::relm4::gtk::{self, prelude::*};
/// kurage::kurage_gen_macros!();
///
/// // Here as an example, you can just redefine the macro for customization
/// macro_rules! kurage_page_pre {
///     () => {
///         use kurage::relm4::prelude::*;
///     };
/// }
/// // if you are going to use the macros outside of the module, export it
/// pub(crate) use kurage_page_pre;
///
/// generate_generator! { generate_page => [<$name Page>] {}:
///   //                  ━━━━━━━━━┯━━━    ╍╍╍╍╍╍╍╍╍╍╍╍╍╍ ┄┄╍
///   //                           │              ┃       ╰─╂── further optional additional fields
///   //                           │              ┣━━━━━━━━━┛   in the new struct
///   //  required new macro name ─┘     (optional) format of new component names
///
///   // writing `init: {}` is totally optional
///   init: {
///     // available metavariables:
///     // - $name
///     // - $modelname (the init variable used for `.launch()`ing this component)
///     // - $root, $initsender, $initmodel, $initwidgets
///   }
///
///   // writing `update: {} => {}` is also optional. Note that all macros generated by
///   // `generate_generator!` will always force `Output` to be a new enum.
///   update: {
///     // available metavariables:
///     // - $self
///     // - $message
///     // - $sender
///
///     // let's try to create a new Input variant
///     Print(s: String) => println!("{s}"),
///   } => {}
///
///   // The original code was:
///   //
///   // libhelium::ViewMono {
///   //   append = &gtk::Box {
///   //     set_orientation: gtk::Orientation::Vertical,
///   //     set_spacing: 4,
///   //
///   //     KURAGE_INNER
///   //   },
///   // },
///   //
///   // For the sake of demonstration and pulling in less build-dependencies for our doc-tests,
///   // let's use only gtk
///
///   gtk::Window {
///     #[wrap(Some)]
///     set_child = &gtk::Box {
///       set_orientation: gtk::Orientation::Vertical,
///       set_spacing: 4,
///
///       KURAGE_INNER // ← this is where the magic happens!
///     }
///   }
/// }
/// pub(crate) use generate_page;
///
/// // let's try using the new macro!
/// generate_page!(Meow:
///   update(self, message, sender) {} => {}
///
///   gtk::Label {
///     set_label: "Hello, World!",
///   },
///   gtk::Button {
///     connect_clicked => Self::Input::Print("Hello!".into()),
///   }
/// );
///
/// mod above_expands_to_this {
///   use super::*;
///   use kurage::relm4::{self, prelude::*};
///   use kurage::relm4::gtk::{self, prelude::*};
///   kurage::generate_component!(MeowPage:
///     update(self, message, sender) {
///       Print(s: String) => println!("{s}"),
///     } => {}
///
///     gtk::Window {
///       #[wrap(Some)]
///       set_child = &gtk::Box {
///         set_orientation: gtk::Orientation::Vertical,
///         set_spacing: 4,
///
///         // here, `KURAGE_INNER` is replaced by the input to `generate_page!`
///         gtk::Label {
///           set_label: "Hello, World!",
///         },
///         gtk::Button {
///           connect_clicked => Self::Input::Print("Hello!".into()),
///         }
///       }
///     }
///   );
/// }
/// ```
///
/// [Taidan]: https://github.com/Ultramarine-Linux/taidan
#[allow(clippy::missing_panics_doc)]
#[proc_macro]
pub fn generate_generator(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let GenerateGeneratorSyn {
        macroname,
        component,
        structblk,
        initblk,
        updateblk,
        updateout,
        view_first,
        views,
    } = syn::parse_macro_input!(input as GenerateGeneratorSyn);
    let views: TokenStream = recurse_replace(views.unwrap(), "KURAGE_INNER", &mut |_| {
        Box::new(quote::quote! { $($viewtt)* }.into_iter())
    })
    .collect();
    let component =
        component.unwrap_or_else(|| quote::quote! { [<$name>] }.into_iter().next().unwrap());
    let structblk = structblk.iter();
    let inputblk = initblk.iter();
    let updateblk = updateblk.iter();
    let updateout = updateout.iter();
    quote::quote! {
        macro_rules! #macroname {
            ($name:ident $({$($model:tt)+})? $(as $modelname:ident)?:
                $(
                $(preinit $preinit:block)?
                init$([$($local_ref:tt)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $initblock:block
                )?
                update($self:ident, $message:ident, $sender:ident) {
                    $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
                }
                => {$( $out:pat ),*}
                $($viewtt:tt)*
            ) => { ::kurage::paste::paste! {
                ::kurage::generate_component!(
                    #component$({#(#structblk)* $($model)+})? $(as $modelname)?:
                    $(init$([$($local_ref)+])?($root, $initsender, $initmodel, $initwidgets) {
                        #(#inputblk)*
                        $initblock
                    })?
                    update($self, $message, $sender) {
                        #(#updateblk)*
                        $( $msg$(($($param: $paramtype),+))? => $msghdl),*
                    } => {#(#updateout)* $($out),*}

                    #view_first #views
                );
            }};
        }
    }.into()
}

/// # Panics
/// No.
#[proc_macro_attribute]
pub fn mangle_ident(
    attr: proc_macro::TokenStream,
    body: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ident = syn::parse_macro_input!(attr as syn::Ident);
    let mut ii = None;
    let body: TokenStream = recurse_replace(body.into(), &ident.to_string(), &mut |i| {
        if ii.is_none() {
            ii = Some(i);
        }
        Box::new(std::iter::once(TokenTree::Ident(ii.clone().unwrap())))
    })
    .collect();
    quote::quote! { #body }.into()
}
