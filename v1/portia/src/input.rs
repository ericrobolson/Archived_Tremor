#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Tick(u32),
    ApplicationExit,
    AssetLoaded { file: &'static str },
    Mouse(Mouse),
    Window(Window),
    Key { key: Key, state: PressState },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PressState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Space,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Window {
    DroppedFile(std::path::PathBuf),
    Resized { height: u32, width: u32 },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mouse {
    Clicked { x: i32, y: i32, button: MouseButton },
    Released { x: i32, y: i32, button: MouseButton },
    Moved { x: i32, y: i32 },
}
