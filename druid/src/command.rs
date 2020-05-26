// Copyright 2019 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Custom commands.

use std::any::{self, Any};
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{WidgetId, WindowId};

pub(crate) type SelectorSymbol = &'static str;

/// An identifier for a particular command.
///
/// The type parameter `T` specifies the commands payload type.
///
/// This should be a unique string identifier. Certain `Selector`s are defined
/// by druid, and have special meaning to the framework; these are listed in the
/// [`druid::commands`] module.
///
/// [`druid::commands`]: commands/index.html
#[derive(Debug, PartialEq, Eq)]
pub struct Selector<T = ()>(SelectorSymbol, PhantomData<T>);

/// An arbitrary command.
///
/// A `Command` consists of a [`Selector`], that indicates what the command is
/// and what type of payload it carries, as well as the actual payload.
///
/// If the payload can't or shouldn't be cloned,
/// wrapping it with [`SingleUse`] allows you to `take` the object.
///
/// # Examples
/// ```
/// use druid::{Command, Selector};
///
/// let selector = Selector::new("process_rows");
/// let rows = vec![1, 3, 10, 12];
/// let command = Command::new(selector, rows);
///
/// assert_eq!(command.get(selector), Some(&vec![1, 3, 10, 12]));
/// ```
///
/// [`Command::new`]: #method.new
/// [`Command::get_object`]: #method.get_object
/// [`SingleUse`]: struct.SingleUse.html
/// [`Selector`]: struct.Selector.html
#[derive(Debug, Clone)]
pub struct Command {
    selector: SelectorSymbol,
    object: Arc<dyn Any>,
}

/// A wrapper type for [`Command`] arguments that should only be used once.
///
/// This is useful if you have some resource that cannot be
/// cloned, and you wish to send it to another widget.
///
/// # Examples
/// ```
/// use druid::{Command, Selector, SingleUse};
///
/// struct CantClone(u8);
///
/// let selector = Selector::new("use-once");
/// let num = CantClone(42);
/// let command = Command::new(selector, SingleUse::new(num));
///
/// let object: &SingleUse<CantClone> = command.get_unchecked(selector);
/// if let Some(num) = object.take() {
///     // now you own the data
///     assert_eq!(num.0, 42);
/// }
///
/// // subsequent calls will return `None`
/// assert!(object.take().is_none());
/// ```
///
/// [`Command`]: struct.Command.html
pub struct SingleUse<T>(Mutex<Option<T>>);

/// The target of a command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    /// The target is the top-level application.
    Global,
    /// The target is a window; the event will be delivered to all
    /// widgets in that window.
    Window(WindowId),
    /// The target is a specific widget.
    Widget(WidgetId),
}

/// Commands with special meaning, defined by druid.
///
/// See [`Command`] for more info.
///
/// [`Command`]: ../struct.Command.html
pub mod sys {
    use super::Selector;
    use crate::{FileDialogOptions, FileInfo, SingleUse};
    use std::any::Any;

    /// Quit the running application. This command is handled by the druid library.
    pub const QUIT_APP: Selector = Selector::new("druid-builtin.quit-app");

    /// Hide the application. (mac only?)
    pub const HIDE_APPLICATION: Selector = Selector::new("druid-builtin.menu-hide-application");

    /// Hide all other applications. (mac only?)
    pub const HIDE_OTHERS: Selector = Selector::new("druid-builtin.menu-hide-others");

    /// The selector for a command to create a new window.
    pub(crate) const NEW_WINDOW: Selector<SingleUse<Box<dyn Any>>> =
        Selector::new("druid-builtin.new-window");

    /// The selector for a command to close a window.
    ///
    /// The command must target a specific window.
    /// When calling `submit_command` on a `Widget`s context, passing `None` as target
    /// will automatically target the window containing the widget.
    pub const CLOSE_WINDOW: Selector = Selector::new("druid-builtin.close-window");

    /// Close all windows.
    pub const CLOSE_ALL_WINDOWS: Selector = Selector::new("druid-builtin.close-all-windows");

    /// The selector for a command to bring a window to the front, and give it focus.
    ///
    /// The command must target a specific window.
    /// When calling `submit_command` on a `Widget`s context, passing `None` as target
    /// will automatically target the window containing the widget.
    pub const SHOW_WINDOW: Selector = Selector::new("druid-builtin.show-window");

    /// Display a context (right-click) menu. The argument must be the [`ContextMenu`].
    /// object to be displayed.
    ///
    /// [`ContextMenu`]: ../struct.ContextMenu.html
    pub(crate) const SHOW_CONTEXT_MENU: Selector<Box<dyn Any>> =
        Selector::new("druid-builtin.show-context-menu");

    /// The selector for a command to set the window's menu. The argument should
    /// be a [`MenuDesc`] object.
    ///
    /// [`MenuDesc`]: ../struct.MenuDesc.html
    pub(crate) const SET_MENU: Selector<Box<dyn Any>> = Selector::new("druid-builtin.set-menu");

    /// Show the application preferences.
    pub const SHOW_PREFERENCES: Selector = Selector::new("druid-builtin.menu-show-preferences");

    /// Show the application about window.
    pub const SHOW_ABOUT: Selector = Selector::new("druid-builtin.menu-show-about");

    /// Show all applications.
    pub const SHOW_ALL: Selector = Selector::new("druid-builtin.menu-show-all");

    /// Show the new file dialog.
    pub const NEW_FILE: Selector = Selector::new("druid-builtin.menu-file-new");

    /// System command. A file picker dialog will be shown to the user, and an
    /// [`OPEN_FILE`] command will be sent if a file is chosen.
    ///
    /// [`OPEN_FILE`]: constant.OPEN_FILE.html
    /// [`FileDialogOptions`]: ../struct.FileDialogOptions.html
    pub const SHOW_OPEN_PANEL: Selector<FileDialogOptions> =
        Selector::new("druid-builtin.menu-file-open");

    /// Open a file, must be handled by the application.
    ///
    /// [`FileInfo`]: ../struct.FileInfo.html
    pub const OPEN_FILE: Selector<FileInfo> = Selector::new("druid-builtin.open-file-path");

    /// Special command. When issued, the system will show the 'save as' panel,
    /// and if a path is selected the system will issue a [`SAVE_FILE`] command
    /// with the selected path as the argument.
    ///
    /// The argument should be a [`FileDialogOptions`] object.
    ///
    /// [`SAVE_FILE`]: constant.SAVE_FILE.html
    /// [`FileDialogOptions`]: ../struct.FileDialogOptions.html
    pub const SHOW_SAVE_PANEL: Selector<FileDialogOptions> =
        Selector::new("druid-builtin.menu-file-save-as");

    /// Save the current file, must be handled by the application.
    ///
    /// How this should be handled depends on the payload:
    /// `Some(handle)`: the app should save to that file and store the `handle` for future use.
    /// `None`: the app should have received `Some` before and use the stored `FileInfo`.
    pub const SAVE_FILE: Selector<Option<FileInfo>> = Selector::new("druid-builtin.menu-file-save");

    /// Show the print-setup window.
    pub const PRINT_SETUP: Selector = Selector::new("druid-builtin.menu-file-print-setup");

    /// Show the print dialog.
    pub const PRINT: Selector = Selector::new("druid-builtin.menu-file-print");

    /// Show the print preview.
    pub const PRINT_PREVIEW: Selector = Selector::new("druid-builtin.menu-file-print");

    /// Cut the current selection.
    pub const CUT: Selector = Selector::new("druid-builtin.menu-cut");

    /// Copy the current selection.
    pub const COPY: Selector = Selector::new("druid-builtin.menu-copy");

    /// Paste.
    pub const PASTE: Selector = Selector::new("druid-builtin.menu-paste");

    /// Undo.
    pub const UNDO: Selector = Selector::new("druid-builtin.menu-undo");

    /// Redo.
    pub const REDO: Selector = Selector::new("druid-builtin.menu-redo");
}

impl Selector {
    /// A selector that does nothing.
    pub const NOOP: Selector = Selector::new("");
}

impl<T> Selector<T> {
    /// Create a new `Selector` with the given string.
    pub const fn new(s: &'static str) -> Selector<T> {
        Selector(s, PhantomData)
    }

    pub(crate) const fn symbol(&self) -> SelectorSymbol {
        self.0
    }
}

impl<T: Any> Selector<T> {
    pub fn carry(self, object: T) -> Command {
        Command::new(self, object)
    }
}

impl Command {
    /// Create a new `Command` with an argument. If you do not need
    /// an argument, `Selector` implements `Into<Command>`.
    pub fn new<T: Any>(selector: Selector<T>, object: T) -> Self {
        Command {
            selector: selector.symbol(),
            object: Arc::new(object),
        }
    }

    /// Used to create a command from the types sent via an `ExtEventSink`.
    pub(crate) fn from_ext(selector: SelectorSymbol, object: Arc<dyn Any>) -> Self {
        Command { selector, object }
    }

    /// Checks if this was created using `selector`.
    pub fn is<T>(&self, selector: Selector<T>) -> bool {
        self.selector == selector.symbol()
    }

    /// Returns `Some(reference)` to this `Command`'s object, if the selector matches.
    ///
    /// Returns `None` when `self.is(selector) == false`.
    ///
    /// If the selector has already been checked, [`get_unchecked`] can be used safely.
    ///
    /// # Panics
    ///
    /// Panics when the payload has a different type, than what the selector is supposed to carry.
    /// This can happen when two selectors with different types but the same key are used.
    ///
    /// [`get_unchecked`]: #method.get_unchecked
    pub fn get<T: Any>(&self, selector: Selector<T>) -> Option<&T> {
        if self.selector == selector.symbol() {
            Some(self.object.downcast_ref().unwrap_or_else(|| {
                panic!(
                    "The selector \"{}\" exists twice with different types. See druid::Command::get_object for more information",
                    selector.symbol()
                )
            }))
        } else {
            None
        }
    }

    /// Return a reference to this `Command`'s object.
    ///
    /// If the selector has already been checked, `get_unchecked` can be used safely.
    /// Otherwise you should either use [`get`] instead, or check the selector using [`is`] first.
    ///
    /// # Panics
    ///
    /// Panics when `self.is(selector) == false`.
    ///
    /// Panics when the payload has a different type, than what the selector is supposed to carry.
    /// This can happen when two selectors with different types but the same key are used.
    ///
    /// [`is`]: #method.is
    /// [`get`]: #method.get
    pub fn get_unchecked<T: Any>(&self, selector: Selector<T>) -> &T {
        self.get(selector).unwrap_or_else(|| {
            panic!(
                "Expected selector \"{}\" but the command was \"{}\".",
                selector.symbol(),
                self.selector
            )
        })
    }
}

impl<T: Any> SingleUse<T> {
    pub fn new(data: T) -> Self {
        SingleUse(Mutex::new(Some(data)))
    }

    /// Takes the value, leaving a None in its place.
    pub fn take(&self) -> Option<T> {
        self.0.lock().unwrap().take()
    }
}

impl From<Selector> for Command {
    fn from(selector: Selector) -> Command {
        Command {
            selector: selector.symbol(),
            object: Arc::new(()),
        }
    }
}

impl<T> std::fmt::Display for Selector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Selector(\"{}\", {})", self.0, any::type_name::<T>())
    }
}

// This has do be done explicitly, to avoid the Copy bound on `T`.
// See https://doc.rust-lang.org/std/marker/trait.Copy.html#how-can-i-implement-copy .
impl<T> Copy for Selector<T> {}
impl<T> Clone for Selector<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl From<WindowId> for Target {
    fn from(id: WindowId) -> Target {
        Target::Window(id)
    }
}

impl From<WidgetId> for Target {
    fn from(id: WidgetId) -> Target {
        Target::Widget(id)
    }
}

impl Into<Option<Target>> for WindowId {
    fn into(self) -> Option<Target> {
        Some(Target::Window(self))
    }
}

impl Into<Option<Target>> for WidgetId {
    fn into(self) -> Option<Target> {
        Some(Target::Widget(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_object() {
        let sel = Selector::new("my-selector");
        let objs = vec![0, 1, 2];
        let command = Command::new(sel, objs);
        assert_eq!(command.get(sel), Some(&vec![0, 1, 2]));
    }
}
