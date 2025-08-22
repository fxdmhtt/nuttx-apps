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
