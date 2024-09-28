use slint::ComponentHandle;

slint::include_modules!();

pub struct MainWindow {
    window: AppWindow,
}

impl MainWindow {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = AppWindow::new()?;

        Ok(Self { window })
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.window.run()
    }
}

slint::slint! {
    import { Button, VerticalBox, HorizontalBox, GroupBox } from "std-widgets.slint";

    component SidebarButton inherits Button {
        width: 40px;
        height: 40px;
    }

    export component AppWindow inherits Window {
        title: "Nodian";
        width: 1024px;
        height: 768px;

        HorizontalBox {
            VerticalBox {
                width: 50px;
                SidebarButton { text: "📝"; }
                SidebarButton { text: "📅"; }
                SidebarButton { text: "🔧"; }
                SidebarButton { text: "📋"; }
            }
            VerticalBox {
                Text {
                    text: "Welcome to Nodian";
                    font-size: 24px;
                    horizontal-alignment: center;
                }
            }
        }
    }
}