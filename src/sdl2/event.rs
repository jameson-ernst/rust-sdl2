/*!
Event Handling
 */

use std::ffi::{c_str_to_bytes};
use std::mem;
use libc::{c_int, c_void, uint32_t};
use std::num::FromPrimitive;
use std::ptr;
use std::borrow::ToOwned;

use controller;
use controller::{ControllerAxis, ControllerButton};
use joystick;
use joystick::HatState;
use keyboard;
use keyboard::Mod;
use sys::keycode::SDL_Keymod;
use keycode::KeyCode;
use mouse;
use mouse::{Mouse, MouseState};
use scancode::ScanCode;
use video;
use get_error;
use SdlResult;

use sys::event as ll;

/// Types of events that can be delivered.
#[derive(Copy, Clone, FromPrimitive)]
#[repr(u32)]
pub enum EventType {
    First = ll::SDL_FIRSTEVENT,

    Quit = ll::SDL_QUIT,
    AppTerminating = ll::SDL_APP_TERMINATING,
    AppLowMemory = ll::SDL_APP_LOWMEMORY,
    AppWillEnterBackground = ll::SDL_APP_WILLENTERBACKGROUND,
    AppDidEnterBackground = ll::SDL_APP_DIDENTERBACKGROUND,
    AppWillEnterForeground = ll::SDL_APP_WILLENTERFOREGROUND,
    AppDidEnterForeground = ll::SDL_APP_DIDENTERFOREGROUND,

    Window = ll::SDL_WINDOWEVENT,
    // TODO: SysWM = ll::SDL_SYSWMEVENT,

    KeyDown = ll::SDL_KEYDOWN,
    KeyUp = ll::SDL_KEYUP,
    TextEditing = ll::SDL_TEXTEDITING,
    TextInput = ll::SDL_TEXTINPUT,

    MouseMotion = ll::SDL_MOUSEMOTION,
    MouseButtonDown = ll::SDL_MOUSEBUTTONDOWN,
    MouseButtonUp = ll::SDL_MOUSEBUTTONUP,
    MouseWheel = ll::SDL_MOUSEWHEEL,

    JoyAxisMotion = ll::SDL_JOYAXISMOTION,
    JoyBallMotion = ll::SDL_JOYBALLMOTION,
    JoyHatMotion = ll::SDL_JOYHATMOTION,
    JoyButtonDown = ll::SDL_JOYBUTTONDOWN,
    JoyButtonUp = ll::SDL_JOYBUTTONUP,
    JoyDeviceAdded = ll::SDL_JOYDEVICEADDED,
    JoyDeviceRemoved = ll::SDL_JOYDEVICEREMOVED,

    ControllerAxisMotion = ll::SDL_CONTROLLERAXISMOTION,
    ControllerButtonDown = ll::SDL_CONTROLLERBUTTONDOWN,
    ControllerButtonUp = ll::SDL_CONTROLLERBUTTONUP,
    ControllerDeviceAdded = ll::SDL_CONTROLLERDEVICEADDED,
    ControllerDeviceRemoved = ll::SDL_CONTROLLERDEVICEREMOVED,
    ControllerDeviceRemapped = ll::SDL_CONTROLLERDEVICEREMAPPED,

    FingerDown = ll::SDL_FINGERDOWN,
    FingerUp = ll::SDL_FINGERUP,
    FingerMotion = ll::SDL_FINGERMOTION,
    DollarGesture = ll::SDL_DOLLARGESTURE,
    DollarRecord = ll::SDL_DOLLARRECORD,
    MultiGesture = ll::SDL_MULTIGESTURE,

    ClipboardUpdate = ll::SDL_CLIPBOARDUPDATE,
    DropFile = ll::SDL_DROPFILE,

    User = ll::SDL_USEREVENT,
    Last = ll::SDL_LASTEVENT,
}

#[derive(Copy, Clone, Show)]
/// An enum of window events.
pub enum WindowEventId {
    None,
    Shown,
    Hidden,
    Exposed,
    Moved,
    Resized,
    SizeChanged,
    Minimized,
    Maximized,
    Restored,
    Enter,
    Leave,
    FocusGained,
    FocusLost,
    Close,
}

impl WindowEventId {
    fn from_ll(id: u8) -> WindowEventId {
        match id {
            1  => WindowEventId::Shown,
            2  => WindowEventId::Hidden,
            3  => WindowEventId::Exposed,
            4  => WindowEventId::Moved,
            5  => WindowEventId::Resized,
            6  => WindowEventId::SizeChanged,
            7  => WindowEventId::Minimized,
            8  => WindowEventId::Maximized,
            9  => WindowEventId::Restored,
            10 => WindowEventId::Enter,
            11 => WindowEventId::Leave,
            12 => WindowEventId::FocusGained,
            13 => WindowEventId::FocusLost,
            14 => WindowEventId::Close,
            _  => WindowEventId::None
        }
    }
}

/// Different event types.
pub enum Event {
    None,

    /// (timestamp)
    Quit(u32),
    AppTerminating(u32),
    AppLowMemory(u32),
    AppWillEnterBackground(u32),
    AppDidEnterBackground(u32),
    AppWillEnterForeground(u32),
    AppDidEnterForeground(u32),

    /// (timestamp, window, winEventId, data1, data2)
    Window(u32, video::Window, WindowEventId, i32, i32),
    // TODO: SysWMEvent

    /// (timestamp, window, keycode, scancode, keymod, repeat)
    KeyDown(u32, video::Window, KeyCode, ScanCode, Mod, bool),
    KeyUp(u32, video::Window, KeyCode, ScanCode, Mod, bool),
    /// (timestamp, window, text, start, length)
    TextEditing(u32, video::Window, String, i32, i32),
    /// (timestamp, window, text)
    TextInput(u32, video::Window, String),

    /// (timestamp, window, which, [MouseState], x, y, xrel, yrel)
    MouseMotion(u32, video::Window, u32, MouseState, i32, i32, i32, i32),
    /// (timestamp, window, which, MouseBtn, x, y)
    MouseButtonDown(u32, video::Window, u32, Mouse, i32, i32),
    MouseButtonUp(u32, video::Window, u32, Mouse, i32, i32),
    /// (timestamp, window, whichId, x, y)
    MouseWheel(u32, video::Window, u32, i32, i32),

    /// (timestamp, whichId, axisIdx, value)
    JoyAxisMotion(u32, i32, u8, i16),
    /// (timestamp, whichId, ballIdx, xrel, yrel)
    JoyBallMotion(u32, i32, u8, i16, i16),
    /// (timestamp, whichId, hatIdx, state)
    JoyHatMotion(u32, i32, u8, HatState),
    /// (timestamp, whichId, buttonIdx)
    JoyButtonDown(u32, i32, u8),
    JoyButtonUp(u32, i32, u8),
    /// (timestamp, whichId)
    JoyDeviceAdded(u32, i32),
    JoyDeviceRemoved(u32, i32),

    /// (timestamp, whichId, axis, value)
    ControllerAxisMotion(u32, i32, ControllerAxis, i16),
    /// (timestamp, whichId, button)
    ControllerButtonDown(u32, i32, ControllerButton),
    ControllerButtonUp(u32, i32, ControllerButton),
    /// (timestamp, whichIdx)
    ControllerDeviceAdded(u32, i32),
    ControllerDeviceRemoved(u32, i32),
    ControllerDeviceRemapped(u32, i32),

    /// (timestamp, touchId, fingerId, x, y, dx, dy, pressure)
    FingerDown(u32, i64, i64, f32, f32, f32, f32, f32),
    FingerUp(u32, i64, i64, f32, f32, f32, f32, f32),
    FingerMotion(u32, i64, i64, f32, f32, f32, f32, f32),

    /// (timestamp, touchId, gestureId, numFingers, error, x, y)
    DollarGesture(u32, i64, i64, u32, f32, f32, f32),
    DollarRecord(u32, i64, i64, u32, f32, f32, f32),
    /// (timestamp, touchId, dTheta, dDist, x, y, numFingers)
    MultiGesture(u32, i64, f32, f32, f32, f32, u16),

    /// (timestamp)
    ClipboardUpdate(u32),

    /// (timestamp, filename)
    DropFile(u32, String),

    /// (timestamp, Window, type, code)
    User(u32, video::Window, u32, i32),
}

impl ::std::fmt::Show for Event {
    fn fmt(&self, out: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        out.write_str(match *self {
            Event::None => "Event::None",
            Event::Quit(..) => "Event::Quit",
            Event::AppTerminating(..) => "Event::AppTerminating",
            Event::AppLowMemory(..) => "Event::AppLowMemory",
            Event::AppWillEnterBackground(..) => "Event::AppWillEnterBackground",
            Event::AppDidEnterBackground(..) => "Event::AppDidEnterBackground",
            Event::AppWillEnterForeground(..) => "Event::AppWillEnterForeground",
            Event::AppDidEnterForeground(..) => "Event::AppDidEnterForeground",
            Event::Window(..) => "Event::Window",
            Event::KeyDown(..) => "Event::KeyDown",
            Event::KeyUp(..) => "Event::KeyUp",
            Event::TextEditing(..) => "Event::TextEditing",
            Event::TextInput(..) => "Event::TextInput",
            Event::MouseMotion(..) => "Event::MouseMotion",
            Event::MouseButtonDown(..) => "Event::MouseButtonDown",
            Event::MouseButtonUp(..) => "Event::MouseButtonUp",
            Event::MouseWheel(..) => "Event::MouseWheel",
            Event::JoyAxisMotion(..) => "Event::JoyAxisMotion",
            Event::JoyBallMotion(..) => "Event::JoyBallMotion",
            Event::JoyHatMotion(..) => "Event::JoyHatMotion",
            Event::JoyButtonDown(..) => "Event::JoyButtonDown",
            Event::JoyButtonUp(..) => "Event::JoyButtonUp",
            Event::JoyDeviceAdded(..) => "Event::JoyDeviceAdded",
            Event::JoyDeviceRemoved(..) => "Event::JoyDeviceRemoved",
            Event::ControllerAxisMotion(..) => "Event::ControllerAxisMotion",
            Event::ControllerButtonDown(..) => "Event::ControllerButtonDown",
            Event::ControllerButtonUp(..) => "Event::ControllerButtonUp",
            Event::ControllerDeviceAdded(..) => "Event::ControllerDeviceAdded",
            Event::ControllerDeviceRemoved(..) => "Event::ControllerDeviceRemoved",
            Event::ControllerDeviceRemapped(..) => "Event::ControllerDeviceRemapped",
            Event::FingerDown(..) => "Event::FingerDown",
            Event::FingerUp(..) => "Event::FingerUp",
            Event::FingerMotion(..) => "Event::FingerMotion",
            Event::DollarGesture(..) => "Event::DollarGesture",
            Event::DollarRecord(..) => "Event::DollarRecord",
            Event::MultiGesture(..) => "Event::MultiGesture",
            Event::ClipboardUpdate(..) => "Event::ClipboardUpdate",
            Event::DropFile(..) => "Event::DropFile",
            Event::User(..) => "Event::User",
        })
    }
}

// TODO: Remove this when from_utf8 is updated in Rust
impl Event {
    fn to_ll(self) -> Option<ll::SDL_Event> {
        let ret = null_event();
        match self {
            // just ignore timestamp
            Event::User(_, ref win, typ, code) => {
                let event = ll::SDL_UserEvent {
                    _type: typ as uint32_t,
                    timestamp: 0,
                    windowID: win.get_id(),
                    code: code as i32,
                    data1: ptr::null(),
                    data2: ptr::null(),
                };
                unsafe {
                    ptr::copy_memory(mem::transmute::<_,*mut ll::SDL_UserEvent>(&ret), &event, 1);
                }
                Some(ret)
            },
            _ => {
                // don't know how to convert!
                None
            }
        }
    }

    fn from_ll(raw: &ll::SDL_Event) -> Event {
        let raw_type = raw._type();
        let raw_type = if raw_type.is_null() {
            return Event::None;
        } else {
            unsafe { *raw_type }
        };

        // if event type has not been defined, treat it as a UserEvent
        let event_type: EventType = FromPrimitive::from_uint(raw_type as usize).unwrap_or(EventType::User);
        unsafe { match event_type {
            EventType::Quit => {
                let ref event = *raw.quit();
                Event::Quit(event.timestamp)
            }
            EventType::AppTerminating => {
                let ref event = *raw.common();
                Event::AppTerminating(event.timestamp)
            }
            EventType::AppLowMemory => {
                let ref event = *raw.common();
                Event::AppLowMemory(event.timestamp)
            }
            EventType::AppWillEnterBackground => {
                let ref event = *raw.common();
                Event::AppWillEnterBackground(event.timestamp)
            }
            EventType::AppDidEnterBackground => {
                let ref event = *raw.common();
                Event::AppDidEnterBackground(event.timestamp)
            }
            EventType::AppWillEnterForeground => {
                let ref event = *raw.common();
                Event::AppWillEnterForeground(event.timestamp)
            }
            EventType::AppDidEnterForeground => {
                let ref event = *raw.common();
                Event::AppDidEnterForeground(event.timestamp)
            }

            EventType::Window => {
                let ref event = *raw.window();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::Window(event.timestamp, window,
                              WindowEventId::from_ll(event.event),
                              event.data1, event.data2)
            }
            // TODO: SysWMEventType

            EventType::KeyDown => {
                let ref event = *raw.key();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::KeyDown(event.timestamp, window,
                               FromPrimitive::from_i32(event.keysym.sym)
                                 .unwrap_or(KeyCode::Unknown),
                               FromPrimitive::from_u32(event.keysym.scancode)
                                 .unwrap_or(ScanCode::Unknown),
                               keyboard::Mod::from_bits(event.keysym._mod as SDL_Keymod).unwrap(),
                               event.repeat != 0)
            }
            EventType::KeyUp => {
                let ref event = *raw.key();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::KeyUp(event.timestamp, window,
                             FromPrimitive::from_i32(event.keysym.sym)
                               .unwrap_or(KeyCode::Unknown),
                             FromPrimitive::from_u32(event.keysym.scancode)
                               .unwrap_or(ScanCode::Unknown),
                             keyboard::Mod::from_bits(event.keysym._mod as SDL_Keymod).unwrap(),
                             event.repeat != 0)
            }
            EventType::TextEditing => {
                let ref event = *raw.edit();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                let text = String::from_utf8_lossy(
                        event.text.iter()
                            .take_while(|&b| (*b) != 0i8)
                            .map(|&b| b as u8)
                            .collect::<Vec<u8>>()
                            .as_slice()
                    ).to_owned().into_owned();
                Event::TextEditing(event.timestamp, window, text,
                                   event.start, event.length)
            }
            EventType::TextInput => {
                let ref event = *raw.text();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                let text = String::from_utf8_lossy(
                        event.text.iter()
                            .take_while(|&b| (*b) != 0i8)
                            .map(|&b| b as u8)
                            .collect::<Vec<u8>>()
                            .as_slice()
                    ).to_owned().into_owned();
                Event::TextInput(event.timestamp, window, text)
            }

            EventType::MouseMotion => {
                let ref event = *raw.motion();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::MouseMotion(event.timestamp, window, event.which,
                                   mouse::MouseState::from_bits_truncate(event.state),
                                   event.x, event.y,
                                   event.xrel, event.yrel)
            }
            EventType::MouseButtonDown => {
                let ref event = *raw.button();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::MouseButtonDown(event.timestamp, window, event.which,
                                       mouse::wrap_mouse(event.button),
                                       event.x, event.y)
            }
            EventType::MouseButtonUp => {
                let ref event = *raw.button();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::MouseButtonUp(event.timestamp, window, event.which,
                                     mouse::wrap_mouse(event.button),
                                     event.x, event.y)
            }
            EventType::MouseWheel => {
                let ref event = *raw.wheel();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::MouseWheel(event.timestamp, window, event.which,
                                  event.x, event.y)
            }

            EventType::JoyAxisMotion => {
                let ref event = *raw.jaxis();
                Event::JoyAxisMotion(event.timestamp, event.which,
                                     event.axis, event.value)
            }
            EventType::JoyBallMotion => {
                let ref event = *raw.jball();
                Event::JoyBallMotion(event.timestamp, event.which,
                                     event.ball, event.xrel, event.yrel)
            }
            EventType::JoyHatMotion => {
                let ref event = *raw.jhat();
                Event::JoyHatMotion(event.timestamp, event.which, event.hat,
                                    joystick::HatState::from_bits(event.value).unwrap())
            }
            EventType::JoyButtonDown => {
                let ref event = *raw.jbutton();
                Event::JoyButtonDown(event.timestamp, event.which, event.button)
            }
            EventType::JoyButtonUp => {
                let ref event = *raw.jbutton();
                Event::JoyButtonUp(event.timestamp, event.which, event.button)
            }
            EventType::JoyDeviceAdded => {
                let ref event = *raw.jdevice();
                Event::JoyDeviceAdded(event.timestamp, event.which)
            }
            EventType::JoyDeviceRemoved => {
                let ref event = *raw.jdevice();
                Event::JoyDeviceRemoved(event.timestamp, event.which)
            }

            EventType::ControllerAxisMotion => {
                let ref event = *raw.caxis();
                let axis = controller::wrap_controller_axis(event.axis);

                Event::ControllerAxisMotion(event.timestamp, event.which,
                                            axis, event.value)
            }
            EventType::ControllerButtonDown => {
                let ref event = *raw.cbutton();
                let button = controller::wrap_controller_button(event.button);

                Event::ControllerButtonDown(event.timestamp, event.which, button)
            }
            EventType::ControllerButtonUp => {
                let ref event = *raw.cbutton();
                let button = controller::wrap_controller_button(event.button);

                Event::ControllerButtonUp(event.timestamp, event.which, button)
            }
            EventType::ControllerDeviceAdded => {
                let ref event = *raw.cdevice();
                Event::ControllerDeviceAdded(event.timestamp, event.which)
            }
            EventType::ControllerDeviceRemoved => {
                let ref event = *raw.cdevice();
                Event::ControllerDeviceRemoved(event.timestamp, event.which)
            }
            EventType::ControllerDeviceRemapped => {
                let ref event = *raw.cdevice();
                Event::ControllerDeviceRemapped(event.timestamp, event.which)
            }

            EventType::FingerDown => {
                let ref event = *raw.tfinger();
                Event::FingerDown(event.timestamp, event.touchId,
                                  event.fingerId, event.x,
                                  event.y, event.dx,
                                  event.dy, event.pressure)
            }
            EventType::FingerUp => {
                let ref event = *raw.tfinger();
                Event::FingerUp(event.timestamp, event.touchId,
                                event.fingerId, event.x,
                                event.y, event.dx,
                                event.dy, event.pressure)
            }
            EventType::FingerMotion => {
                let ref event = *raw.tfinger();
                Event::FingerMotion(event.timestamp,
                                    event.touchId, event.fingerId,
                                    event.x, event.y,
                                    event.dx, event.dy,
                                    event.pressure)
            }
            EventType::DollarGesture => {
                let ref event = *raw.dgesture();
                Event::DollarGesture(event.timestamp, event.touchId,
                                     event.gestureId, event.numFingers,
                                     event.error, event.x, event.y)
            }
            EventType::DollarRecord => {
                let ref event = *raw.dgesture();
                Event::DollarRecord(event.timestamp, event.touchId,
                                    event.gestureId, event.numFingers,
                                    event.error, event.x, event.y)
            }
            EventType::MultiGesture => {
                let ref event = *raw.mgesture();
                Event::MultiGesture(event.timestamp, event.touchId,
                                    event.dTheta, event.dDist,
                                    event.x, event.y, event.numFingers)
            }

            EventType::ClipboardUpdate => {
                let ref event = *raw.common();
                Event::ClipboardUpdate(event.timestamp)
            }
            EventType::DropFile => {
                let ref event = *raw.drop();

                let buf = c_str_to_bytes(&event.file);
                let text = String::from_utf8_lossy(buf).to_string();
                ll::SDL_free(event.file as *const c_void);

                Event::DropFile(event.timestamp, text)
            }

            EventType::First | EventType::Last => Event::None,

            // If we have no other match and the event type is >= 32768
            // this is a user event
            EventType::User => {
                if raw_type < 32768 {
                    return Event::None;
                }

                let ref event = *raw.user();

                let window = video::Window::from_id(event.windowID);
                let window = match window {
                    Err(_) => return Event::None,
                    Ok(window) => window,
                };

                Event::User(event.timestamp, window, raw_type, event.code)
            }
        }}                      // close unsafe & match


    }
}

fn null_event() -> ll::SDL_Event {
    ll::SDL_Event { data: [0; 56] }
}

/// Pump the event loop, gathering events from the input devices.
pub fn pump_events() {
    unsafe { ll::SDL_PumpEvents(); }
}

/// Check for the existence of certain event types in the event queue.
pub fn has_event(_type: EventType) -> bool {
    unsafe { ll::SDL_HasEvent(_type as uint32_t ) == 1 }
}

/// Check for the existence of a range of event types in the event queue.
pub fn has_events(min: EventType, max: EventType) -> bool {
    unsafe { ll::SDL_HasEvents(min as uint32_t, max as uint32_t) == 1 }
}

/// Clear events from the event queue.
pub fn flush_event(_type: EventType) {
    unsafe { ll::SDL_FlushEvent(_type as uint32_t) }
}

/// Clear events from the event queue of a range of event types.
pub fn flush_events(min: EventType, max: EventType) {
    unsafe { ll::SDL_FlushEvents(min as uint32_t, max as uint32_t) }
}

/// Poll for currently pending events.
pub fn poll_event() -> Event {
    pump_events();

    let raw = null_event();
    let success = unsafe { ll::SDL_PollEvent(&raw) == 1 as c_int };

    if success { Event::from_ll(&raw) }
    else { Event::None }
}

/// Wait indefinitely for the next available event.
pub fn wait_event() -> SdlResult<Event> {
    let raw = null_event();
    let success = unsafe { ll::SDL_WaitEvent(&raw) == 1 as c_int };

    if success { Ok(Event::from_ll(&raw)) }
    else { Err(get_error()) }
}

/// Wait until the specified timeout (in milliseconds) for the next available event.
pub fn wait_event_timeout(timeout: i32) -> SdlResult<Event> {
    let raw = null_event();
    let success = unsafe { ll::SDL_WaitEventTimeout(&raw, timeout as c_int) ==
                           1 as c_int };

    if success { Ok(Event::from_ll(&raw)) }
    else { Err(get_error()) }
}

extern "C" fn event_filter_wrapper(userdata: *const c_void, event: *const ll::SDL_Event) -> c_int {
    let filter: extern fn(event: Event) -> bool = unsafe { mem::transmute(userdata) };
    if event.is_null() { 1 }
    else { filter(Event::from_ll(unsafe { &*event })) as c_int }
}

/// Set up a filter to process all events before they change internal state and are posted to the internal event queue.
pub fn set_event_filter(filter_func: extern fn(event: Event) -> bool) {
    unsafe { ll::SDL_SetEventFilter(event_filter_wrapper,
                                    filter_func as *const _) }
}

/// Add a callback to be triggered when an event is added to the event queue.
pub fn add_event_watch(filter_func: extern fn(event: Event) -> bool) {
    unsafe { ll::SDL_AddEventWatch(event_filter_wrapper,
                                   filter_func as *const _) }
}

/// Remove an event watch callback added.
pub fn delete_event_watch(filter_func: extern fn(event: Event) -> bool) {
    unsafe { ll::SDL_DelEventWatch(event_filter_wrapper,
                                   filter_func as *const _) }
}

/// Run a specific filter function on the current event queue, removing any events for which the filter returns 0.
pub fn filter_events(filter_func: extern fn(event: Event) -> bool) {
    unsafe { ll::SDL_FilterEvents(event_filter_wrapper,
                                  filter_func as *const _) }
}

/// Set the state of processing events.
pub fn set_event_state(_type: EventType, state: bool) {
    unsafe { ll::SDL_EventState(_type as uint32_t,
                                state as ll::SDL_EventState); }
}

/// Get the state of processing events.
pub fn get_event_state(_type: EventType) -> bool {
    unsafe { ll::SDL_EventState(_type as uint32_t, ll::SDL_QUERY)
             == ll::SDL_ENABLE }
}

/// allocate a set of user-defined events, and return the beginning event number for that set of events
pub fn register_events(num_events: i32) -> Option<u32> {
    let ret = unsafe { ll::SDL_RegisterEvents(num_events as c_int) };
    if ret == (-1 as uint32_t) {
        None
    } else {
        Some(ret as u32)
    }
}

/// add an event to the event queue
pub fn push_event(event: Event) -> SdlResult<()> {
    match event.to_ll() {
        Some(raw_event) => {
            let ok = unsafe { ll::SDL_PushEvent(&raw_event) == 1 };
            if ok { Ok(()) }
            else { Err(get_error()) }
        },
        None => {
            Err("Unsupport event type to push back to queue.".to_owned())
        }
    }
}
