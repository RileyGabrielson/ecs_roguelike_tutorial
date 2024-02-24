use rltk::{Rltk, VirtualKeyCode, RGB};

pub fn create_character(ctx: &mut Rltk, items: Vec<String>) -> Option<String> {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Choose an item",
    );

    for (index, item) in items.iter().enumerate() {
        ctx.print_color_centered(
            17 + (index * 2),
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            format!("{}: {}", index + 1, item),
        );
    }

    match ctx.key {
        None => None,
        Some(key) => match key {
            VirtualKeyCode::Key1 => items.get(0).cloned(),
            VirtualKeyCode::Key2 => items.get(1).cloned(),
            VirtualKeyCode::Key3 => items.get(2).cloned(),
            VirtualKeyCode::Key4 => items.get(3).cloned(),
            VirtualKeyCode::Key5 => items.get(4).cloned(),
            VirtualKeyCode::Key6 => items.get(5).cloned(),
            VirtualKeyCode::Key7 => items.get(6).cloned(),
            VirtualKeyCode::Key8 => items.get(7).cloned(),
            VirtualKeyCode::Key9 => items.get(8).cloned(),
            _ => None,
        },
    }
}
