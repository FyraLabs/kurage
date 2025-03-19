pub use paste;
pub use relm4;
#[cfg(feature = "tracing")]
pub use tracing;

#[macro_export]
macro_rules! generate_pages {
    ($Page:ident $AppModel:ident $AppMsg:ident: $($num:tt: $page:ident $($forward:expr)?),+$(,)?) => { $crate::paste::paste! {
        use pages::{$([<_$num _$page:lower>]::[<$page:camel Page>]),+};
        use pages::{$([<_$num _$page:lower>]::[<$page:camel PageOutput>]),+};


        #[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
        pub enum $Page {
            #[default]
            $([< $page:camel >]),+
        }

        impl TryFrom<usize> for $Page {
            type Error = ();

            fn try_from(value: usize) -> Result<Self, ()> {
                #[allow(clippy::zero_prefixed_literal)]
                Ok(match value {
                    $( $num => Self::[<$page:camel>], )+
                    _ => return Err(()),
                })
            }
        }
        impl From<$Page> for usize {
            fn from(val: $Page) -> Self {
                #[allow(clippy::zero_prefixed_literal)]
                match val {
                    $( $Page::[<$page:camel>] => $num, )+
                }
            }
        }

        #[derive(Debug)]
        pub struct $AppModel {
            page: $Page,
            $(
                pub [<$page:snake _page>]: $crate::relm4::Controller<[<$page:camel Page>]>,
            )+
        }

        impl $AppModel {
            fn _default(sender: ComponentSender<Self>) -> Self {Self {
                page: $Page::default(),
                $(
                    [<$page:snake _page>]: [<$page:camel Page>]::builder()
                        .launch(())
                        .forward(sender.input_sender(), $crate::generate_pages!(@$page $AppMsg $($forward)?)),
                )+
            }}
        }
    }};
    (@$page:ident $AppMsg:ident) => { $crate::paste::paste! {
        |msg| match msg {
            [<$page:camel PageOutput>]::Nav(action) => $AppMsg::Nav(action),
        }
    }};
    (@$page:ident $AppMsg:ident $forward:expr) => { $forward };
}

/// Generate a [`relm4::SimpleComponent`].
///
/// This expands to
/// - declaration of the model struct (`struct MyLabel { … }`)
///   - this comes with `#[derive(Debug, Derive)]`
/// - declaration of `enum MyLabelMsg { … }` for `type Input = MyLabelMsg;`
/// - impl `fn init()` if the optional `init()` is omitted in macro invocation
/// - `fn update()` alongside the declaration of the variants in `MyLabelMsg`
/// - declaration of `enum MyLabelOutput { … }` for `type Output = MyLabelOutput` (unless another
///   type is specified otherwise)
///
/// # Examples
///
/// Here is an over-engineered way to show a button labelled "Hello, World!". When the button is
/// clicked, "Hello, World!" is printed in the console.
///
/// ```
/// use kurage_macro_rules::generate_component;
/// use relm4::prelude::*;
/// use relm4::gtk::{self, prelude::*};
/// generate_component!(MyLabel:
///   update(self, message, sender) {} => {}
///   //                            ┯━    ┬─
///   //          enum Self::Input ─╯     │
///   //                                  │
///   // enum Self::Output (you can also specify your own types)
///
///   gtk::Label {
///     set_label: "Hello, World!",
///   },
/// );
///
/// #[derive(Debug)] // required for all fields of MyOtherComponent
/// struct MyOtherComponentInner {
///   mylabel: relm4::component::Connector<MyLabel>,
/// }
/// impl Default for MyOtherComponentInner {
///   fn default() -> Self {
///     Self { mylabel: MyLabel::builder().launch(()) }
///   }
/// }
///
/// generate_component!(MyOtherComponent {
///   btn: gtk::Button,
///   inner: MyOtherComponentInner,
/// }:
///   init[btn](root, sender, model, widgets) /* for my_init_var: MyInitType */ {
///     //──┬──                               ══════════════════════════════╤══
///     //  ╰─ optional, a space separated list of things for #[local_ref]  │
///     //                                                                  ╵
///     //     when you would like to set anything other than `type Init = ();`
///     //
///     // NOTE: init() is entirely optional, but update() is required.
///     //
///     // code in this block will be run only after `view_output!()`.
///     // also, the model (i.e. `self`) is initialized using `Self::default()`.
///
///     model.inner.mylabel.detach_runtime(); // model is mutable
///   }
///   update(self, message, sender) {
///     ButtonClicked => println!("Hello, World!"),
///   } => {}
///
///   gtk::Box {
///     #[local_ref] btn ->
///     gtk::Button {
///       connect_clicked => Self::Input::ButtonClicked,
///
///       set_child: Some(model.inner.mylabel.widget()),
///     }
///   }
/// );
/// ```
#[macro_export]
macro_rules! generate_component {
    ($comp:ident $({$($model:tt)+})?:
        $(
        $(preinit $preinit:block)?
        init$([$($local_ref:tt)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $(for $init:ident: $InitType:ty)? $initblock:block
        )?
        update($self:ident, $message:ident, $sender:ident) {
            $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
        }
        => $out:tt
        $($viewtt:tt)*
    ) => { $crate::paste::paste! {
        $crate::generate_component!{ @model $comp $($($model)+)?}
        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub enum [<$comp Msg>] {
            $($msg$(($($paramtype),+))?),*
        }

        $crate::generate_component!(@out $comp $out);

        // HACK: this ensures `#[watch]` is parsed correctly for `model` idents
        #[::kurage::mangle_ident(model)]
        $(#[::kurage::mangle_ident($initmodel)])?
        #[$crate::relm4::component(pub)]
        impl $crate::relm4::SimpleComponent for $comp {
            #[allow(unused_parens)]
            type Init = ($($($InitType)?)?);
            type Input = [<$comp Msg>];
            type Output = $crate::generate_component!(@outty $comp $out);

            #[allow(clippy::used_underscore_binding)]
            #[allow(unused_variables)]
            fn init(
                init: Self::Init,
                root: Self::Root,
                $sender: $crate::relm4::ComponentSender<Self>,
            ) -> $crate::relm4::ComponentParts<Self> {
                #[allow(unused_mut)]
                let mut model = Self::default();
                $(
                    #[allow(unused_mut, unused_assignments)]
                    let mut $initmodel = model;

                    $($crate::generate_component!(@localref $initmodel $($local_ref)+))?;

                    let $root = root.clone();
                    $(let $init = init;)?
                    $($preinit;)?
                )?

                // HACK: invoking view_output!() directly gives `()` when $init* is given.
                // I don't know why this fixes the issue. — mado
                let widgets = [<view _output>]!();

                $(
                // HACK: this solves variable name obfuscation in macro_rules! {}
                let $initwidgets = widgets;
                #[allow(unused_variables)]
                let $initsender = $sender;

                $initblock

                let model = $initmodel;
                let widgets = $initwidgets;
                )?

                $crate::relm4::ComponentParts { model, widgets }
            }

            fn update(&mut $self, $message: Self::Input, $sender: $crate::relm4::ComponentSender<Self>) {
                // tracing::trace!(?$message, "{}", concat!(stringify!($comp), ": received message"));
                match $message {
                    $(Self::Input::$msg$(($($param),+))? => $msghdl),*
                }
            }

            view! { $($viewtt)* }
        }
    }};
    (@model $comp:ident $($model:tt)+) => { $crate::paste::paste! {
        #[derive(Debug, Default)]
        pub struct [<$comp>] {$($model)+}
    }};
    (@model $comp:ident) => { $crate::paste::paste! {
        #[derive(Debug, Default)]
        pub struct [<$comp>];
    }};
    (@out $comp:ident {$( $out:tt )*}) => { $crate::paste::paste! {
        #[derive(Debug)]
        pub enum [<$comp Output>] {
            $($out)*
        }
    }};
    (@out $comp:ident $outty:ty) => { };
    (@outty $comp:ident {$( $out:tt )*}) => { $crate::paste::paste! { [<$comp Output>] }};
    (@outty $comp:ident $outty:ty) => { $outty };
    (@localref $initmodel:ident) => {};
    (@localref $initmodel:ident $local_ref:ident {$($inner:tt)+} $($next:tt)*) => {
        let $local_ref = {$($inner)+};
        $crate::generate_component!(@localref $initmodel $($next)*);
    };
    (@localref $initmodel:ident $local_ref:ident $nextident:ident $($next:tt)*) => {
        let $local_ref = &$initmodel.$local_ref;
        $crate::generate_component!(@localref $initmodel $nextident $($next)*);
    };
    (@localref $initmodel:ident $local_ref:ident) => {
        let $local_ref = &$initmodel.$local_ref;
    };
}

/// Macros used by other 🪼 macros.
///
/// You should execute this macro somewhere in your codebase.
/// This generates a `mod kurage_generated_macros` that contains `pub(crate)`-exported macros that
/// are used by other 🪼 macros.
///
/// The reason these macros exist separately is that you may override these macros freely in order
/// to customize the behaviour of the macros. These are documented in `Customization` sections of
/// the corresponding macros.
#[macro_export]
macro_rules! kurage_gen_macros {
    () => {
        #[allow(unused_macros)]
        mod kurage_generated_macros {
            macro_rules! kurage_page_pre {
                () => {
                    use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
                };
            }
            pub(crate) use kurage_page_pre;
        }
    };
}
