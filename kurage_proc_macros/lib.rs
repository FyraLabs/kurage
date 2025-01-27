/// Parses the following syntax:
///
/// ```ignore
/// generate_generator! { name =>
///    gtk::Label { ... },
///    Idk { whatelse: true },
/// }
/// ```
struct GenerateGeneratorSyn {
    macroname: syn::Ident,
    component: Option<proc_macro2::TokenTree>,
    views: proc_macro2::TokenStream,
}

impl syn::parse::Parse for GenerateGeneratorSyn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let macroname = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let rest = input.parse()?;
        if input.peek(syn::Token![=>]) {
            input.parse::<syn::Token![=>]>()?;
            let views = input.parse()?;
            Ok(Self {
                macroname,
                component: Some(rest),
                views,
            })
        } else {
            Ok(Self {
                macroname,
                component: None,
                views: rest.into(),
            })
        }
    }
}

fn recurse_replace_kurage_inner(
    ts: proc_macro2::TokenStream,
) -> impl Iterator<Item = proc_macro2::TokenTree> {
    ts.into_iter().flat_map(|tt| match tt {
        proc_macro2::TokenTree::Ident(i) if i == "KURAGE_INNER" => {
            Box::new(quote::quote! { $($viewtt)* }.into_iter())
                as Box<dyn Iterator<Item = proc_macro2::TokenTree>>
        }
        proc_macro2::TokenTree::Group(group) => Box::new(std::iter::once(
            proc_macro2::TokenTree::Group(proc_macro2::Group::new(
                group.delimiter(),
                recurse_replace_kurage_inner(group.stream()).collect(),
            )),
        )),
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
/// Example extracted from [Taidan]: <https://github.com/Ultramarine-Linux/taidan/blob/c146bf06d52ee318aff72d09b5d7e74080f4502f/src/macros.rs#L23C1-L43C30>
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
/// generate_generator! { generate_page => [<$name Page>] =>
///   //                          ━┯━━━━━━━━━━━    ╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ╌╌
///   //  required new macro name ─┘             (optional) format of new component names
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
//
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
/// );
///
/// mod above_expands_to_this {
///   use super::*;
///   use kurage::relm4::{self, prelude::*};
///   use kurage::relm4::gtk::{self, prelude::*};
///   kurage::generate_component!(MeowPage:
///     update(self, message, sender) {} => {}
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
        views,
    } = syn::parse_macro_input!(input as GenerateGeneratorSyn);
    let views: proc_macro2::TokenStream = recurse_replace_kurage_inner(views).collect();
    let component =
        component.unwrap_or_else(|| quote::quote! { [<$name>] }.into_iter().next().unwrap());
    quote::quote! {
        macro_rules! #macroname {
            ($name:ident $({$($model:tt)+})? $(as $modelname:ident)?:
                $(
                init$([$($local_ref:ident)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $initblock:block
                )?
                update($self:ident, $message:ident, $sender:ident) {
                    $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
                }
                => {$( $out:pat ),*}
                $($viewtt:tt)*
            ) => { ::kurage::paste::paste! {
                kurage_page_pre!();
                ::kurage::generate_component!(
                    #component$({$($model)+})? $(as $modelname)?:
                    $(init$([$($local_ref)+])?($root, $initsender, $initmodel, $initwidgets) $initblock)?
                    update($self, $message, $sender) {
                    // Nav(action: NavAction) => $sender.output(Self::Output::Nav(action)).unwrap(),
                    $( $msg$(($($param: $paramtype),+))? => $msghdl),*
                    } => {/*Nav(NavAction),*/ $($out),*}

                    #views
                );
            }};
        }
    }.into()
}
