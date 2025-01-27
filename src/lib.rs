pub mod page;

pub use kurage_proc_macros::*;
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

        // macro_rules! list_pages {
        //     () => {$(page)+};
        // }
        // pub(crate) use list_pages;
    }};
    (@$page:ident $AppMsg:ident) => { $crate::paste::paste! {
        |msg| match msg {
            [<$page:camel PageOutput>]::Nav(action) => $AppMsg::Nav(action),
        }
    }};
    (@$page:ident $AppMsg:ident $forward:expr) => { $forward };
}

#[macro_export]
macro_rules! generate_component {
    ($comp:ident $({$($model:tt)+})?:
        $(
        init$([$($local_ref:ident)+])?($root:ident, $initsender:ident, $initmodel:ident, $initwidgets:ident) $(for $init:ident: $InitType:ty)? $initblock:block
        )?
        update($self:ident, $message:ident, $sender:ident) {
            $( $msg:ident$(($($param:ident: $paramtype:ty),+$(,)?))? => $msghdl:expr ),*$(,)?
        }
        => $out:tt
        $($viewtt:tt)*
    ) => { $crate::paste::paste! {
        $crate::generate_component!{ @model $comp $($($model)+)?}
        #[derive(Clone, Debug)]
        pub enum [<$comp Msg>] {
            $($msg$(($($paramtype),+))?),*
        }

        $crate::generate_component!(@out $comp $out);

        #[relm4::component(pub)]
        impl SimpleComponent for $comp {
            #[allow(unused_parens)]
            type Init = ($($($InitType)?)?);
            type Input = [<$comp Msg>];
            type Output = $crate::generate_component!(@outty $comp $out);

            view! { $($viewtt)* }


            #[allow(clippy::used_underscore_binding)]
            #[allow(unused_variables)]
            fn init(
                init: Self::Init,
                root: Self::Root,
                $sender: ComponentSender<Self>,
            ) -> ComponentParts<Self> {
                $crate::generate_component!(@default {
                    let model = Self::default();
                } $({
                    #[allow(unused_mut, unused_assignments)]
                    let mut $initmodel = Self::default();
                })?);

                $($($(let $local_ref = &$initmodel.$local_ref;)+)?)?

                $(
                    let $root = root.clone();
                    $(let $init = init;)?
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

                ComponentParts { model, widgets }
            }

            fn update(&mut $self, $message: Self::Input, $sender: ComponentSender<Self>) {
                tracing::trace!(?$message, "{}", concat!(stringify!($comp), ": received message"));
                match $message {
                    $(Self::Input::$msg$(($($param),+))? => $msghdl),*
                }
            }
        }
    }};
    (@model $comp:ident $($model:tt)+) => { $crate::paste::paste! {
        #[derive(Debug, Default)]
        pub struct [<$comp>] {$($model)+}
    }};
    (@model $comp:ident) => {paste::paste! {
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

    (@default {$($default:tt)+} {$($if:tt)+}) => { $($if)+ };
    (@default {$($default:tt)+}) => { $($default)+ };

    (@init $body:block {$($inner:tt)+}($sender:ident)) => {
        fn init(
            init: Self::Init,
            root: Self::Root,
            $sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> { $body }
    };
    (@init $body:block {$($inner:tt)+}($sender:ident $root:ident)) => {
        fn init(
            init: Self::Init,
            $root: Self::Root,
            $sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> { $body }
    };
    (@init $body:block {$($inner:tt)+}($sender:ident $root:ident $init:ident)) => {
        fn init(
            $init: Self::Init,
            $root: Self::Root,
            $sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> { $body }
    };
    (@do_nothing $($tt:tt)+) => { $($tt:tt)+ };
}

/// Macros used by other kurage macros.
///
/// You may override these macros freely.
#[macro_export]
macro_rules! kurage_gen_macros {
    () => {
        $crate::page::gen_macros!();
    };
}

#[macro_export]
macro_rules! dollar {
    () => {};
}
