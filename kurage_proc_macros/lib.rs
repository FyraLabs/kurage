/// Parses the following syntax:
///
/// ```no_run
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

#[proc_macro]
pub fn generate_generator(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let GenerateGeneratorSyn {
        macroname,
        component,
        views,
    } = syn::parse_macro_input!(input as GenerateGeneratorSyn);
    let views: proc_macro2::TokenStream = recurse_replace_kurage_inner(views).collect();
    let component = component.unwrap_or(quote::quote! { [<$name>] }.into_iter().next().unwrap());
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
            ) => { kurage::paste::paste! {
                kurage_page_pre!();
                kurage::generate_component!(
                    #component$({$($model)+})? $(as $modelname)?:
                    $(init$([$($local_ref)+])?($root, $initsender, $initmodel, $initwidgets) $initblock)?
                    update($self, $message, $sender) {
                    Nav(action: NavAction) => $sender.output(Self::Output::Nav(action)).unwrap(),
                    $( $msg$(($($param: $paramtype),+))? => $msghdl),*
                    } => {Nav(NavAction), $($out),*}

                    #views
                );
            }};
        }
    }.into()
}
