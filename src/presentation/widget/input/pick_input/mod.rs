pub(super) mod optional;
pub(super) mod required;

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use iced::widget::{Id, combo_box};

use crate::presentation::Viewable;
struct PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    id: Id,
    options: combo_box::State<T>,
    selection: Option<T>,
    placeholder: String,
    message: Rc<dyn Fn(Id, T) -> Message>,
}

impl<T, Message> Viewable<Message> for PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq + 'static,
    Message: Clone + 'static,
{
    fn view(&self) -> iced::Element<'_, Message> {
        let message = self.message.clone();
        let id = self.id.clone();
        combo_box(
            &self.options,
            &self.placeholder,
            self.selection.as_ref(),
            move |input| message(id.clone(), input),
        )
        .into()
    }
}

impl<T, Message> PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    pub(super) fn id(&self) -> &Id {
        &self.id
    }
}

impl<T, Message> Debug for PickInput<T, Message>
where
    T: Debug + Clone + Display + PartialEq,
    Message: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PickInput")
            .field("id", &self.id)
            .field("options", &self.options)
            .field("selections", &self.selection)
            .finish()
    }
}
