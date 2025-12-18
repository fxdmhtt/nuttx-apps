#[macro_export]
macro_rules! clone {
    ( $( $var:ident ),* ) => {
        $(
            let $var = $var.clone();
        )*
    };
}

#[macro_export]
macro_rules! downgrade {
    ( $( $var:ident ),* ) => {
        $(
            let $var = std::rc::Rc::downgrade(&$var);
        )*
    };
}

#[macro_export]
macro_rules! BindingText {
    ($obj:expr, $body:block) => {
        reactive_cache::effect!(|| {
            let obj = match $obj.try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            let val = $body;
            unsafe { lv_label_set_text(obj, val.as_ptr() as _) };
        })
    };
}

#[macro_export]
macro_rules! BindingSliderValue {
    ($obj:expr, $signal:expr, $event:ident, Convert $Convert:expr, ConvertBack $ConvertBack:expr) => {{
        $crate::runtime::lvgl::event::add(&$obj, $event, |e| {
            let obj = unsafe { lv_event_get_target(e) };
            let val = unsafe { lv_bar_get_value(obj) } as u8; // Refer to `lv_slider_get_value`
            let val = $ConvertBack(val);
            $signal.set(val);
        });

        reactive_cache::effect!(|| {
            let obj = match $obj.try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            let val = *$signal.get();
            let val = $Convert(val);
            unsafe { lv_bar_set_value(obj, val.into(), LV_ANIM_OFF) }; // Refer to `lv_slider_set_value`
        })
    }};
    ($obj:expr, $signal:expr, Convert $Convert:expr, ConvertBack $ConvertBack:expr) => {
        BindingSliderValue!($obj, $signal, LV_EVENT_VALUE_CHANGED, Convert |v| v, ConvertBack $ConvertBack)
    };
    ($obj:expr, $signal:expr, ConvertBack $ConvertBack:expr) => {
        BindingSliderValue!($obj, $signal, Convert |v| v, ConvertBack $ConvertBack)
    };
    ($obj:expr, $signal:expr, Convert $Convert:expr) => {
        BindingSliderValue!($obj, $signal, Convert $Convert, ConvertBack |v| v)
    };
    ($obj:expr, $signal:expr) => {{
        BindingSliderValue!($obj, $signal, Convert |v| v, ConvertBack |v| v)
    }};
}

#[macro_export]
macro_rules! BindingStyle {
    ($obj:expr, $part:ident, $setter:ident, $body:block) => {
        reactive_cache::effect!(|| {
            let obj = match $obj.try_get() {
                Ok(obj) => obj,
                Err(_) => return,
            };
            let val = $body;
            unsafe { $setter(obj, val, $part) };
        })
    };
}

#[macro_export]
macro_rules! BindingStyleMacroMaker {
    ($name:ident, $func:ident) => {
        #[macro_export]
        macro_rules! $name {
            ($obj:expr, $part:ident, $body:block) => {
                $crate::BindingStyle!($obj, $part, $func, $body)
            };
            ($obj:expr, $body:block) => {
                $name!($obj, LV_PART_MAIN, $body)
            };
        }
    };
}

// BindingStyleMacroMaker!(BindingBgColor, lv_obj_set_style_bg_color);
// BindingStyleMacroMaker!(BindingImageRecolor, lv_obj_set_style_image_recolor);
// BindingStyleMacroMaker!(BindingImageRecolorOpa, lv_obj_set_style_image_recolor_opa);

#[macro_export]
macro_rules! BindingBgColor {
    ($obj:expr, $part:ident, $body:block) => {
        $crate::BindingStyle!($obj, $part, lv_obj_set_style_bg_color, $body)
    };
    ($obj:expr, $body:block) => {
        BindingBgColor!($obj, LV_PART_MAIN, $body)
    };
}

#[macro_export]
macro_rules! BindingImageRecolor {
    ($obj:expr, $part:ident, $body:block) => {
        $crate::BindingStyle!($obj, $part, lv_obj_set_style_image_recolor, $body)
    };
    ($obj:expr, $body:block) => {
        BindingImageRecolor!($obj, LV_PART_MAIN, $body)
    };
}

#[macro_export]
macro_rules! BindingImageRecolorOpa {
    ($obj:expr, $part:ident, $body:block) => {
        $crate::BindingStyle!($obj, $part, lv_obj_set_style_image_recolor_opa, $body)
    };
    ($obj:expr, $body:block) => {
        BindingImageRecolorOpa!($obj, LV_PART_MAIN, $body)
    };
}
