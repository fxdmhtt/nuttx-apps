#[macro_export]
macro_rules! BindingText {
    ($obj:expr, $body:block) => {
        reactive_cache::effect!(|| {
            let obj = $obj;
            let val = $body;
            unsafe { lv_label_set_text(obj, val.as_ptr() as _) };
        })
    };
}

#[macro_export]
macro_rules! BindingSliderValue {
    ($obj:expr, $signal:ident, Convert $Convert:expr, ConvertBack $ConvertBack:expr) => {{
        $crate::runtime::event::add($obj, LV_EVENT_VALUE_CHANGED, |e| {
            let obj = unsafe { lv_event_get_target(e) };
            let value = unsafe { lv_bar_get_value(obj) } as u8; // Refer to `lv_slider_get_value`
            let val = $ConvertBack(value);
            ${concat($signal, _set)}(val);
        });

        reactive_cache::effect!(|| {
            let obj = $obj;
            let value = *${concat($signal, _get)}();
            let val = $Convert(value);
            unsafe { lv_bar_set_value(obj, val.into(), LV_ANIM_OFF) }; // Refer to `lv_slider_set_value`
        })
    }};
    ($obj:expr, $signal:ident, ConvertBack $ConvertBack:expr) => {
        BindingSliderValue!($obj, $signal, Convert |v| v, ConvertBack $ConvertBack)
    };
    ($obj:expr, $signal:ident, Convert $Convert:expr) => {
        BindingSliderValue!($obj, $signal, Convert $Convert, ConvertBack |v| v)
    };
    ($obj:expr, $signal:ident) => {{
        BindingSliderValue!($obj, $signal, Convert |v| v, ConvertBack |v| v)
    }};
}

#[macro_export]
macro_rules! BindingStyle {
    ($obj:expr, $part:ident, $setter:ident, $body:block) => {
        reactive_cache::effect!(|| {
            let obj = $obj;
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
