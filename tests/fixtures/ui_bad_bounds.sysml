// UI bounds violation fixture — should trigger UI001
// Elements that exceed the 128x64 display resolution

package Firmware {

    import Firmware::*;

    // ── Display: 128x64 ──

    part def SmallDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::i2c;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display";
        }
    }

    // ── Valid font and icon so only bounds checking fails ──

    part def Font5x8 {
        @FontAsset {
            family = "fixed";
            size = 8;
            source = AssetSource::builtin;
            file = "";
        }
    }

    part def AlertIcon {
        @IconAsset {
            width = 8;
            height = 8;
            source = AssetSource::file;
            format = IconFormat::mono_1bpp;
            file = "assets/icons/alert.raw";
        }
    }

    // ── Screen with elements that exceed display bounds ──

    part def OverflowScreen {
        @Screen {
            display = "small_display";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }

        // FAIL: x(120) + width(16) = 136 > 128
        part def overflow_right {
            @Element {
                type = ElementKind::icon;
                x = 120;
                y = 0;
                width = 16;
                height = 16;
                font = "";
                icon = "alert_icon";
                align = AlignKind::left;
                scroll = false;
                truncate = false;
                binding_module = "";
                binding_field = "";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }

        // FAIL: y(60) + height(8) = 68 > 64
        part def overflow_bottom {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 60;
                width = 8;
                height = 8;
                font = "font_5x8";
                icon = "";
                align = AlignKind::left;
                scroll = false;
                truncate = false;
                binding_module = "";
                binding_field = "";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }

        // PASS: fully within bounds (0+64=64 <= 128, 0+32=32 <= 64)
        part def within_bounds {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 64;
                height = 32;
                font = "font_5x8";
                icon = "";
                align = AlignKind::center;
                scroll = false;
                truncate = true;
                binding_module = "";
                binding_field = "";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }
    }
}
