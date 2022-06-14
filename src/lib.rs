use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use chrono::{Date, Datelike, Local, Weekday};

// Methods ending with _wtd are used for unit testing and internal implementations. They allow
// supplying today with any date.

/// Custom errors returned by this crate.
#[derive(Debug, PartialEq)]
pub enum MtdError {
    /// Indicates that no `Todo` with the given `id` exists.
    NoTodoWithGivenId(u64),
    /// Indicates that no `Task` with the given `id` exists.
    NoTaskWithGivenId(u64),
}

impl Display for MtdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MtdError::NoTodoWithGivenId(id) => {
                write!(f, "No Todo with the given id: \"{}\" found.", id)
            }
            MtdError::NoTaskWithGivenId(id) => {
                write!(f, "No Task with the given id: \"{}\" found.", id)
            }
        }
    }
}

impl std::error::Error for MtdError {}

/// Gets the date that represents the upcoming weekday. Given tomorrow’s weekday, this should return
/// tomorrows date. Today is represented by the current weekday.
fn weekday_to_date(weekday: Weekday, mut today: Date<Local>) -> Date<Local> {
    loop {
        if today.weekday() == weekday {
            return today;
        }
        today = today.succ();
    }
}

/// Represents a one-time task to be done at a specific date. The date is specified as a weekday
/// from now. If no weekday is given, the current weekday will be used. After the given weekday, the
/// `Todo` will show up for the current day.
#[derive(Debug, PartialEq, Clone)]
pub struct Todo {
    body: String,
    date: Date<Local>,
    id: u64,
    done: Option<Date<Local>>,
    state: ItemState,
}

impl Todo {
    /// Creates a new `Todo` that shows up to be done for the current day.
    pub fn new_undated(body: String) -> Todo {
        Todo {
            body,
            date: Local::today(),
            id: 0,
            done: None,
            state: ItemState::Unchanged,
        }
    }

    /// Creates a new `Todo` that shows up to be done at a specific weekday after which it will show
    /// for the current day.
    pub fn new_dated(body: String, weekday: Weekday) -> Todo {
        Todo {
            body,
            date: weekday_to_date(weekday, Local::today()),
            id: 0,
            done: None,
            state: ItemState::Unchanged,
        }
    }

    // Used for unit testing with non-today dependant date
    #[cfg(test)]
    fn new_specific_date(body: String, date: Date<Local>) -> Todo {
        Todo {
            body,
            date,
            id: 0,
            done: None,
            state: ItemState::Unchanged,
        }
    }

    /// Returns `true` if the `Todo` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Datelike, Local};
    /// use mtd::Todo;
    ///
    /// let todo_for_today = Todo::new_undated("I am for today".to_string());
    ///
    /// assert!(todo_for_today.for_date(Local::today()));
    ///
    /// let todo_for_tomorrow = Todo::new_dated("I am for tomorrow".to_string(), Local::today().succ().weekday());
    ///
    /// assert!(!todo_for_tomorrow.for_date(Local::today()));
    /// assert!(todo_for_tomorrow.for_date(Local::today().succ()));
    /// ```
    pub fn for_date(&self, date: Date<Local>) -> bool {
        self.for_date_wtd(date, Local::today())
    }

    fn for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> bool {
        date >= self.date && (date == today || self.date > today)
    }

    /// Gets the `body` of the `Todo`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the weekday of the `Todo`.
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }

    /// Gets the `id` of the `Todo`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Sets the `body` of the `Todo`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the weekday of the `Todo`.
    pub fn set_weekday(&mut self, weekday: Weekday) {
        self.date = weekday_to_date(weekday, Local::today());
    }

    /// Returns `true` if the `Todo` is done.
    pub fn done(&self) -> bool {
        self.done.is_some()
    }

    /// Sets the done state of the `Todo`.
    pub fn set_done(&mut self, done: bool) {
        self.set_done_wtd(done, Local::today());
    }

    fn set_done_wtd(&mut self, done: bool, today: Date<Local>) {
        if done {
            self.done = Some(today);
        } else {
            self.done = None;
        }
    }

    /// Returns `true` if the `Todo` can be removed. A `Todo` can be removed one day after its
    /// completion.
    pub fn can_remove(&self) -> bool {
        self.can_remove_wtd(Local::today())
    }

    fn can_remove_wtd(&self, today: Date<Local>) -> bool {
        if let Some(done_date) = self.done {
            today > done_date
        } else {
            false
        }
    }
}

impl Display for Todo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.body, self.id)
    }
}

/// Represents a reoccurring task for the given weekday(s).
#[derive(Debug, PartialEq, Clone)]
pub struct Task {
    body: String,
    weekdays: Vec<Weekday>,
    done_map: HashMap<Weekday, Date<Local>>,
    id: u64,
    state: ItemState,
}

impl Task {
    /// Creates a new task for the given weekday(s).
    ///
    /// # Panics
    ///
    /// If the given weekdays list is empty.
    pub fn new(body: String, weekdays: Vec<Weekday>) -> Task {
        if weekdays.is_empty() {
            panic!("Cannot create a task without specifying at least one weekday.")
        }
        Task { body, weekdays, id: 0, done_map: HashMap::new(), state: ItemState::Unchanged }
    }

    /// Gets the `body` of the `Task`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the `weekdays` of the `Task`. Note that duplicate weekdays are allowed.
    pub fn weekdays(&self) -> &Vec<Weekday> {
        &self.weekdays
    }

    /// Gets the `id` of the `Task`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Sets the `body` of the `Task`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the `weekdays` of the `Task`.
    pub fn set_weekdays(&mut self, weekdays: Vec<Weekday>) {
        self.weekdays = weekdays;
    }

    /// Adds a weekday to the weekdays list.
    pub fn add_weekday(&mut self, weekday: Weekday) {
        // It doesn't matter if there are duplicate weekdays.
        self.weekdays.push(weekday);
    }

    /// Removes a weekday from the weekdays list. Removes all duplicates as well.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::Weekday;
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Test task".to_string(), vec![Weekday::Mon, Weekday::Tue, Weekday::Wed]);
    /// task.remove_weekday(Weekday::Wed);
    ///
    /// // Removing a weekday that isn't listed does nothing.
    /// task.remove_weekday(Weekday::Fri);
    ///
    /// assert!(task.weekdays().contains(&Weekday::Mon));
    /// assert!(task.weekdays().contains(&Weekday::Tue));
    /// // Doesn't contain wed anymore
    /// assert!(!task.weekdays().contains(&Weekday::Wed));
    /// ```
    pub fn remove_weekday(&mut self, removed_wd: Weekday) {
        let mut new_weekdays = Vec::new();

        for wd in &self.weekdays {
            if wd != &removed_wd {
                new_weekdays.push(wd.clone());
            }
        }

        self.set_weekdays(new_weekdays);
    }

    /// Returns `true` if the `Task` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let task = Task::new("Task".to_string(), vec![Weekday::Fri, Weekday::Sun]);
    ///
    /// assert!(task.for_date(Local.ymd(2022, 6, 10))); // 2022-6-10 is a Friday
    /// assert!(!task.for_date(Local.ymd(2022, 6, 11))); // Saturday
    /// assert!(task.for_date(Local.ymd(2022, 6, 12))); // Sunday
    /// ```
    pub fn for_date(&self, date: Date<Local>) -> bool {
        self.weekdays.contains(&date.weekday())
    }

    /// Returns `true` if the `Task` is done for the given date. Always returns `true` if the task
    /// is not for the given the date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Task".to_string(), vec![Weekday::Mon, Weekday::Wed, Weekday::Thu]);
    ///
    /// task.set_done(true, Local.ymd(2022, 6, 13));
    /// task.set_done(true, Local.ymd(2022, 6, 16));
    ///
    /// // Done for mon and thu
    /// assert!(task.done(Local.ymd(2022, 6, 13)));
    /// assert!(task.done(Local.ymd(2022, 6, 16)));
    ///
    /// // Not done for wed
    /// assert!(!task.done(Local.ymd(2022, 6, 15)));
    ///
    /// // Not done for the following week's mon/thu
    /// assert!(!task.done(Local.ymd(2022, 6, 20)));
    /// assert!(!task.done(Local.ymd(2022, 6, 23)));
    ///
    /// // Since 2022-6-21 is a tue, the task is done for that date
    /// assert!(task.done(Local.ymd(2022, 6, 21)));
    /// ```
    pub fn done(&self, date: Date<Local>) -> bool {
        if self.for_date(date) {
            if let Some(d) = self.done_map.get(&date.weekday()) {
                return *d >= date;
            }
            return false;
        }
        true
    }


    /// Sets the done state of the `Task` for the given date.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Task".to_string(), vec![Weekday::Mon]);
    ///
    /// task.set_done(true, Local.ymd(2022, 6, 13));
    /// assert!(task.done(Local.ymd(2022, 6, 13)));
    ///
    /// task.set_done(false, Local.ymd(2022, 6, 13));
    /// assert!(!task.done(Local.ymd(2022, 6, 13)));
    /// ```
    pub fn set_done(&mut self, done: bool, date: Date<Local>) {
        if done {
            self.done_map.insert(date.weekday(), date);
        } else {
            self.done_map.remove(&date.weekday());
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.body, self.id)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ItemState {
    New,
    Removed,
    Unchanged,
}

trait SyncItem {
    fn set_state(&mut self, state: ItemState);
    fn state(&self) -> ItemState;
    fn set_id(&mut self, id: u64);
}

impl SyncItem for Todo{
    fn set_state(&mut self, state: ItemState) {
        self.state = state;
    }

    fn state(&self) -> ItemState {
        self.state
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

impl SyncItem for Task {
    fn set_state(&mut self, state: ItemState) {
        self.state = state;
    }

    fn state(&self) -> ItemState {
        self.state
    }

    fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}

#[derive(Debug)]
struct SyncList<T: SyncItem + Clone> {
    items: Vec<T>,
    server: bool,
}

impl<T: SyncItem + Clone> SyncList<T> {
    fn new(server: bool) -> Self {
        Self {
            items: Vec::new(),
            server,
        }
    }
    fn add(&mut self, mut item: T) {
        if self.server {
            item.set_state(ItemState::Unchanged);
        } else {
            item.set_state(ItemState::New);
        }
        item.set_id(self.items.len() as u64);
        self.items.push(item);
    }
    fn mark_removed(&mut self, id: u64) -> Result<(), ()> {
        if let Some(item) = self.items.get_mut(id as usize) {
            if self.server {
                self.items.remove(id as usize);
                self.map_indices_to_ids();
                Ok(())
            } else {
                if item.state() == ItemState::Removed {
                    Err(())
                } else {
                    item.set_state(ItemState::Removed);
                    Ok(())
                }
            }
        } else {
            Err(())
        }
    }
    fn map_indices_to_ids(&mut self) {
        for (new_id, item) in self.items.iter_mut().enumerate() {
            item.set_id(new_id as u64);
        }
    }
    fn items(&self) -> Vec<&T> {
        let mut items = Vec::new();
        for item in &self.items {
            if item.state() != ItemState::Removed {
                items.push(item);
            }
        }

        items
    }
    fn get_item_mut(&mut self, id: u64) -> Option<&mut T> {
        self.items.get_mut(id as usize)
    }
}

/// A synchronizable list used for containing and managing all `Todo`s and `Task`s. `Todo`s and
/// `Task`s have `id`s that match their index within the internal `Vec`s of the `TdList`.
#[derive(Debug)]
pub struct TdList {
    todos: SyncList<Todo>,
    tasks: SyncList<Task>,
    server: bool,
}

impl TdList {
    /// Creates a new empty client `TdList`.
    pub fn new_client() -> Self {
        Self { todos: SyncList::new(false), tasks: SyncList::new(false), server: false }
    }

    /// Creates a new empty server `TdList`.
    pub fn new_server() -> Self {
        Self { todos: SyncList::new(true), tasks: SyncList::new(true), server: true }
    }

    /// Gets all the `Todo`s in the list. A `Todo`'s id matches its index in this list.
    pub fn todos(&self) -> Vec<&Todo> {
        self.todos.items()
    }

    /// Gets all the `Task`s in the list. A `Task`'s id matches its index in this list.
    pub fn tasks(&self) -> Vec<&Task> {
        self.tasks.items()
    }

    /// Adds a `Todo` to the list and modifies its id.
    pub fn add_todo(&mut self, todo: Todo) {
        self.todos.add(todo);
    }

    /// Adds a `Task` to the list and modifies its id.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.add(task)
    }

    /// Removes the `Todo` that matches the given id. The id matches the index of the `Todo`. If no
    /// such `Todo` exists, does nothing. (Note for clients: this method only marks items as removed,
    /// to actually remove them, the TdList must be synchronized)
    pub fn remove_todo(&mut self, id: u64) -> Result<(), MtdError> {
        self.todos.mark_removed(id).map_err(|_| MtdError::NoTodoWithGivenId(id))
    }

    /// Removes the `Task` that matches the given id. The id matches the index of the `Task`. If no
    /// such `Task` exists, does nothing. (Note for clients: this method only marks items as removed,
    /// to actually remove them, the TdList must be synchronized)
    pub fn remove_task(&mut self, id: u64) -> Result<(), MtdError> {
        self.tasks.mark_removed(id).map_err(|_| MtdError::NoTaskWithGivenId(id))
    }

    /// Returns a mutable reference to a `Todo`.
    pub fn get_todo_mut(&mut self, id: u64) -> Option<&mut Todo> {
        self.todos.get_item_mut(id)
    }

    /// Returns a mutable reference to a `Task`.
    pub fn get_task_mut(&mut self, id: u64) -> Option<&mut Task> {
        self.tasks.get_item_mut(id)
    }

    /// Returns all `Todo`s for a given date that are not yet done.
    pub fn undone_todos_for_date(&self, date: Date<Local>) -> Vec<&Todo> {
        self.undone_todos_for_date_wtd(date, Local::today())
    }

    /// Returns all `Todo`s for a given date that are done.
    pub fn done_todos_for_date(&self, date: Date<Local>) -> Vec<&Todo> {
        self.done_todos_for_date_wtd(date, Local::today())
    }

    fn undone_todos_for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> Vec<&Todo> {
        let mut undone_todos = Vec::new();

        for todo in self.todos.items() {
            if todo.for_date_wtd(date, today) && !todo.done() {
                undone_todos.push(todo);
            }
        }

        undone_todos
    }

    fn done_todos_for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> Vec<&Todo> {
        let mut done_todos = Vec::new();

        for todo in self.todos.items() {
            if todo.for_date_wtd(date, today) && todo.done() {
                done_todos.push(todo);
            }
        }

        done_todos
    }

    /// Returns all `Task`s for a given date that are not yet done.
    pub fn undone_tasks_for_date(&self, date: Date<Local>) -> Vec<&Task> {
        let mut undone_tasks = Vec::new();

        for task in self.tasks.items() {
            if task.for_date(date) && !task.done(date) {
                undone_tasks.push(task);
            }
        }

        undone_tasks
    }

    /// Returns all `Task`s for a given date that are done.
    pub fn done_tasks_for_date(&self, date: Date<Local>) -> Vec<&Task> {
        let mut done_tasks = Vec::new();

        for task in self.tasks.items() {
            if task.for_date(date) && task.done(date) {
                done_tasks.push(task);
            }
        }

        done_tasks
    }

    /// Removes all `Todo`s that are done and at least a day has passed since their completion.
    /// Basically remove all `Todo`s which `Todo.can_remove()` returns `true`. This is called
    /// automatically every sync.
    /// (Note for clients: this method only marks items as removed, to actually
    /// remove them, the TdList must be synchronized)
    pub fn remove_old_todos(&mut self) {
        self.remove_old_todos_wtd(Local::today());
    }

    fn remove_old_todos_wtd(&mut self, today: Date<Local>) {
        if self.server {
            // This actually removes items...
            self.todos.items = self.todos.items
                .drain(..)
                .filter(|todo| { !todo.can_remove_wtd(today) })
                .collect()
        } else {
            // ...while this only marks them as removed
            for todo in &mut self.todos.items {
                if todo.can_remove_wtd(today) {
                    todo.state = ItemState::Removed;
                }
            }
        }
    }

    /// Synchronizes the list with itself actually removing items. Synchronizing may change the `id`s
    /// of both `Todo`s and `Task`s. Additionally removes old `Todo`s.
    pub fn self_sync(&mut self) {
        self.remove_old_todos();
        self.todos.items.retain(|todo| todo.state != ItemState::Removed);
        self.tasks.items.retain(|task| task.state != ItemState::Removed);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone, Weekday};

    use crate::{MtdError, Task, TdList, Todo, weekday_to_date};

    // Unit test a private function to remove the need to pass today into the Todo constructor
    #[test]
    fn weekday_to_date_returns_correct_dates() {
        // Today is a Tuesday
        let today = Local.ymd(2022, 6, 7);

        // Tue should return today’s date
        assert_eq!(weekday_to_date(Weekday::Tue, today), today);

        // Wed should return tomorrow’s date
        assert_eq!(weekday_to_date(Weekday::Wed, today), today.succ());

        // Mon should return next weeks monday
        assert_eq!(weekday_to_date(Weekday::Mon, today), Local.ymd(2022, 6, 13));
    }

    #[test]
    fn todo_displays_correctly() {
        let todo = Todo::new_undated("Todo".to_string());
        assert_eq!(todo.to_string(), "Todo (ID: 0)".to_string());
    }

    #[test]
    fn todo_for_date_tests() {
        let todo = Todo::new_specific_date("Friday".to_string(), Local.ymd(2022, 6, 10));

        let today = Local.ymd(2022, 6, 10);

        // The following 5 asserts could each be their own unit test but I'm to lazy to do it so
        // instead I just added some comments explaining the tests

        assert!(todo.for_date_wtd(today, today)); // Todo is for the given date on the same day
        assert!(todo.for_date_wtd(today, today.pred())); // Todo is for the given date before the given date
        assert!(!todo.for_date_wtd(today, today.succ())); // Todo is not for the given date after the given date
        assert!(todo.for_date_wtd(today.succ(), today.succ())); // Todo is for the following date one day after the given date
        assert!(!todo.for_date_wtd(today.succ(), today)); // Todo is not for the following date because it is already for today
    }

    #[test]
    fn todo_can_remove_returns_true_only_after_one_day_from_completion() {
        let mut todo = Todo::new_specific_date("Todo".to_string(), Local.ymd(2022, 4, 25));
        todo.set_done_wtd(true, Local.ymd(2022, 4, 26));

        assert!(!todo.can_remove_wtd(Local.ymd(2022, 4, 26)));
        assert!(todo.can_remove_wtd(Local.ymd(2022, 4, 27)));
        assert!(todo.can_remove_wtd(Local.ymd(2022, 4, 28)));
    }

    #[test]
    #[should_panic]
    fn task_new_panics_if_empty_weekday_vec() {
        Task::new("Panic!".to_string(), vec![]);
    }

    #[test]
    fn task_remove_weekday_removes_all_duplicates() {
        let mut task = Task::new("Test task".to_string(), vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Wed]);

        task.remove_weekday(Weekday::Wed);

        assert!(task.weekdays().contains(&Weekday::Mon));
        assert!(task.weekdays().contains(&Weekday::Tue));
        assert!(!task.weekdays().contains(&Weekday::Wed));
    }

    #[test]
    fn task_displays_correctly() {
        let task = Task::new("Task".to_string(), vec![Weekday::Wed]);
        assert_eq!(task.to_string(), "Task (ID: 0)".to_string());
    }

    #[test]
    fn tdlist_add_todo_updates_ids() {
        let mut list = TdList::new_client();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));
        list.add_todo(Todo::new_undated("Todo 2".to_string()));

        assert_eq!(list.todos()[0].id(), 0);
        assert_eq!(list.todos()[1].id(), 1);
        assert_eq!(list.todos()[2].id(), 2);
    }

    #[test]
    fn tdlist_removed_todos_not_visible() {
        let mut list = TdList::new_client();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));
        list.add_todo(Todo::new_undated("Todo 2".to_string()));

        list.remove_todo(1).unwrap();

        assert_eq!(list.todos()[0].body(), "Todo 0");
        assert_eq!(list.todos()[1].body(), "Todo 2");
        assert_eq!(list.todos().len(), 2);
    }

    #[test]
    fn tdlist_remove_todo_returns_err_nonexistent_id() {
        let mut list = TdList::new_client();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));

        assert_eq!(list.remove_todo(2).err().unwrap(), MtdError::NoTodoWithGivenId(2));
    }

    #[test]
    fn tdlist_add_task_updates_ids() {
        let mut list = TdList::new_client();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 2".to_string(), vec![Weekday::Mon]));

        assert_eq!(list.tasks()[0].id(), 0);
        assert_eq!(list.tasks()[1].id(), 1);
        assert_eq!(list.tasks()[2].id(), 2);
    }

    #[test]
    fn tdlist_removed_tasks_not_visible() {
        let mut list = TdList::new_client();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 2".to_string(), vec![Weekday::Mon]));

        list.remove_task(1).unwrap();

        assert_eq!(list.tasks()[0].body(), "Task 0");
        assert_eq!(list.tasks()[1].body(), "Task 2");
        assert_eq!(list.tasks().len(), 2);
    }

    #[test]
    fn tdlist_remove_task_returns_err_with_nonexistent_id() {
        let mut list = TdList::new_client();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));

        assert_eq!(list.remove_task(2).err().unwrap(), MtdError::NoTaskWithGivenId(2));
    }

    #[test]
    fn tdlist_get_todo_mut_returns_mutable() {
        let mut list = TdList::new_client();

        list.add_todo(Todo::new_undated("Todo".to_string()));

        assert_eq!(list.todos()[0].body(), "Todo");

        list.get_todo_mut(0).unwrap().set_body("To-Do".to_string());

        assert_eq!(list.todos()[0].body(), "To-Do");
    }

    #[test]
    fn tdlist_get_task_mut_returns_mutable() {
        let mut list = TdList::new_client();

        list.add_task(Task::new("Task".to_string(), vec![Weekday::Mon]));

        assert_eq!(list.tasks()[0].body(), "Task");

        list.get_task_mut(0).unwrap().set_body("Ta-Sk".to_string());

        assert_eq!(list.tasks()[0].body(), "Ta-Sk");
    }

    fn tdlist_with_done_and_undone() -> TdList {
        let mut list = TdList::new_client();

        list.add_todo(Todo::new_specific_date("Undone 1".to_string(), Local.ymd(2021, 4, 1)));
        list.add_todo(Todo::new_specific_date("Undone 2".to_string(), Local.ymd(2021, 3, 29)));
        list.add_todo(Todo::new_specific_date("Done 1".to_string(), Local.ymd(2021, 4, 1)));
        list.add_todo(Todo::new_specific_date("Done 2".to_string(), Local.ymd(2021, 3, 30)));

        list.get_todo_mut(2).unwrap().set_done_wtd(true, Local.ymd(2021, 4, 1));
        list.get_todo_mut(3).unwrap().set_done_wtd(true, Local.ymd(2021, 4, 1));

        list.add_task(Task::new("Undone 1".to_string(), vec![Weekday::Thu]));
        list.add_task(Task::new("Done 1".to_string(), vec![Weekday::Thu]));

        list.get_task_mut(1).unwrap().set_done(true, Local.ymd(2021, 4, 1));

        list
    }

    #[test]
    fn tdlist_undone_todos_for_date_returns_only_undone() {
        let list = tdlist_with_done_and_undone();

        let returned = list.undone_todos_for_date_wtd(Local.ymd(2021, 4, 1), Local.ymd(2021, 4, 1));

        assert!(returned.contains(&&list.todos()[0]));
        assert!(returned.contains(&&list.todos()[1]));
        assert!(!returned.contains(&&list.todos()[2]));
        assert!(!returned.contains(&&list.todos()[3]));
        assert_eq!(returned.len(), 2);
    }

    #[test]
    fn tdlist_done_todos_for_date_returns_only_done() {
        let list = tdlist_with_done_and_undone();

        let returned = list.done_todos_for_date_wtd(Local.ymd(2021, 4, 1), Local.ymd(2021, 4, 1));

        assert!(!returned.contains(&&list.todos()[0]));
        assert!(!returned.contains(&&list.todos()[1]));
        assert!(returned.contains(&&list.todos()[2]));
        assert!(returned.contains(&&list.todos()[3]));
        assert_eq!(returned.len(), 2);
    }

    #[test]
    fn tdlist_undone_tasks_for_date_returns_only_undone() {
        let list = tdlist_with_done_and_undone();

        let returned = list.undone_tasks_for_date(Local.ymd(2021, 4, 1));

        assert!(returned.contains(&&list.tasks()[0]));
        assert!(!returned.contains(&&list.tasks()[1]));
        assert_eq!(returned.len(), 1);
    }

    #[test]
    fn tdlist_done_tasks_for_date_returns_only_done() {
        let list = tdlist_with_done_and_undone();

        let returned = list.done_tasks_for_date(Local.ymd(2021, 4, 1));

        assert!(!returned.contains(&&list.tasks()[0]));
        assert!(returned.contains(&&list.tasks()[1]));
        assert_eq!(returned.len(), 1);
    }

    #[test]
    fn tdlist_remove_old_todos_removes_done_after_1_day() {
        let mut list = tdlist_with_done_and_undone();
        let list_containing_same_todos_for_eq_check = tdlist_with_done_and_undone();

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 1));

        assert_eq!(list.todos(), list_containing_same_todos_for_eq_check.todos());

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 2));

        assert_eq!(list.todos()[0], list_containing_same_todos_for_eq_check.todos()[0]);
        assert_eq!(list.todos()[1], list_containing_same_todos_for_eq_check.todos()[1]);
        assert_eq!(list.todos().len(), 2);
    }

    #[test]
    fn tdlist_client_only_self_sync_actually_removes_items() {
        let mut list = tdlist_with_done_and_undone();

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 2));
        list.remove_task(1).unwrap();

        assert_eq!(list.todos.items.len(), 4);
        assert_eq!(list.tasks.items.len(), 2);

        list.self_sync();

        assert_eq!(list.todos.items.len(), 2);
        assert_eq!(list.tasks.items.len(), 1);
    }

    #[test]
    fn tdlist_server_always_removes_items() {
        let mut list = tdlist_with_done_and_undone();
        list.server = true;
        list.todos.server = true;
        list.tasks.server = true;

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 2));
        list.remove_task(1).unwrap();

        assert_eq!(list.todos.items.len(), 2);
        assert_eq!(list.tasks.items.len(), 1);
    }

    #[test]
    fn tdlist_sync_always_removes_old_todos() {
        let mut list = tdlist_with_done_and_undone();

        assert_eq!(list.todos.items.len(), 4);

        list.self_sync();

        assert_eq!(list.todos.items.len(), 2);
    }
}