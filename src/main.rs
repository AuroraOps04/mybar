use gtk::{Application, Window};
use gtk::{Label, prelude::*};
fn main() {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    app.connect_activate(|app| {
        let label = Label::builder().label("FFf").build();
        let window = Window::builder()
            .application(app)
            .title("Hello World")
            .child(&label)
            .build();

        window.present();
    });
    app.run();
}
