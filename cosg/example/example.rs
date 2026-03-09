// examples/hello.rs
// cargo run --example hello

use cosg::*;

fn main() {
    let app = App::new(
        AppConfig {
            title:  "CosG — Hello".into(),
            width:  640,
            height: 480,
            theme:  Theme::violet_dark(),
        },
        || {
            let btn = Button::new()
                .label("Kliknij mnie")
                .esc(Esc::new().border_radius(12.0).opacity(1.0))
                .on_press(|| println!("pressed!"));

            let lbl = Label::new("CosG UI");

            Box::new(
                Container::new()
                    .padding(24.0)
                    .esc(Esc::new().border_radius(0.0))
                    .add(lbl)
                    .add(btn),
            )
        },
    );

    app.run();
}