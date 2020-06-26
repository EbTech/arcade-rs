macro_rules! struct_events {
    (
        keyboard: { $( $k_alias:ident : $k_sdl:ident ),* },
        else: { $( $e_alias:ident : $e_sdl:pat ),* }
    )
    => {
        use sdl2::EventPump;


        pub struct ImmediateEvents {
            $( pub $k_alias : Option<bool> , )*
            $( pub $e_alias : bool , )*
            resize: Option<(u32, u32)>
        }

        impl ImmediateEvents {
            pub fn new() -> ImmediateEvents {
                ImmediateEvents {
                    $( $k_alias: None , )*
                    $( $e_alias: false , )*
                    resize: None
                }
            }
        }


        pub struct Events {
            pump: EventPump,
            pub now: ImmediateEvents,

            $( pub $k_alias: bool ),*
        }

        impl Events {
            pub fn new(pump: EventPump) -> Events {
                Events {
                    pump: pump,
                    now: ImmediateEvents::new(),

                    $( $k_alias: false ),*
                }
            }

            pub fn pump(&mut self, renderer: &mut sdl2::render::WindowCanvas) {
                self.now = ImmediateEvents::new();

                for event in self.pump.poll_iter() {
                    use sdl2::event::Event::*;
                    use sdl2::event::WindowEvent::Resized;
                    use sdl2::keyboard::Keycode::*;

                    match event {
                        KeyDown { keycode, .. } => match keycode {
                            $(
                                Some($k_sdl) => {
                                    // Prevent multiple presses when keeping a key down
                                    // Was previously not pressed?
                                    if !self.$k_alias {
                                        // Key pressed
                                        self.now.$k_alias = Some(true);
                                    }

                                    self.$k_alias = true;
                                }
                            ),*
                            _ => {}
                        },

                        KeyUp { keycode, .. } => match keycode {
                            $(
                                Some($k_sdl) => {
                                    // Key released
                                    self.now.$k_alias = Some(false);
                                    self.$k_alias = false;
                                }
                            ),*
                            _ => {}
                        },

                        $(
                            $e_sdl => {
                                self.now.$e_alias = true;
                            }
                        ),*
                        
                        Window { win_event: Resized(w, h), .. } => {
                            panic!("Decide which of these two lines to use!");
                            //self.now.resize = Some((w, h));
                            self.now.resize = Some(renderer.output_size().unwrap());
                        },

                        _ => {}
                    }
                }
            }
        }
    }
}
