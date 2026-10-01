//! Reading a scenario from its text form.
//!
//! 由其文本形式读取场景。

use crate::error::{ScenarioError, ScenarioErrorKind};
use crate::scenario::Scenario;
use apsis_resource::{Resource, ResourceId, Window};
use apsis_task::{Task, TaskId};

/// Reads a scenario from its text form.
///
/// The format is described on the crate documentation. Parsing happens in
/// three passes, so that a `require` line may name a resource added later in
/// the file and a `precedence` line may name tasks in either order:
///
/// 1. the header and the resources,
/// 2. the tasks and their requirements,
/// 3. the precedence constraints.
///
/// Every failure carries the line it came from, so a reader can be pointed at
/// the text that caused it.
///
/// 由其文本形式读取场景。
///
/// 该格式在 crate 文档中描述。解析分三遍进行，使 `require` 行可以指名
/// 文件中稍后才添加的资源，`precedence` 行可以任意次序指名任务：
///
/// 1. 头部与资源，
/// 2. 任务及其需求，
/// 3. 先后约束。
///
/// 每次失败都携带其所来自的行，使读者可被指向造成它的文本。
pub(crate) fn parse(text: &str) -> Result<Scenario, ScenarioError> {
    let lines = collect_lines(text);
    let mut scenario = parse_resources(&lines)?;
    parse_tasks(&lines, &mut scenario)?;
    parse_precedence(&lines, &mut scenario)?;
    scenario.validate()?;
    Ok(scenario)
}

/// One significant line: its one based number and its whitespace separated
/// fields.
///
/// 一个有意义的行：其以 1 为基的行号及其以空白分隔的字段。
struct Line<'a> {
    number: usize,
    fields: Vec<&'a str>,
}

impl Line<'_> {
    fn keyword(&self) -> &str {
        self.fields[0]
    }

    /// Returns the fields from an index to the end, joined by single spaces.
    ///
    /// This is used for the scenario name, which may contain spaces.
    ///
    /// 返回自某下标到末尾的字段，以单个空格连接。
    ///
    /// 用于场景名称，它可能含有空格。
    fn rest(&self, index: usize) -> String {
        self.fields[index.min(self.fields.len())..].join(" ")
    }

    /// Returns the field at an index, or a missing argument error.
    ///
    /// 返回某下标处的字段，或一个缺少参数的错误。
    fn field(&self, index: usize) -> Result<&str, ScenarioError> {
        self.fields.get(index).copied().ok_or_else(|| {
            ScenarioError::new(ScenarioErrorKind::MissingArgument)
                .with_detail(self.keyword())
                .with_line(self.number)
        })
    }

    fn fuse(&self, index: usize) -> Result<f64, ScenarioError> {
        let text = self.field(index)?;
        text.parse::<f64>().map_err(|_| {
            ScenarioError::new(ScenarioErrorKind::BadArgument)
                .with_detail(format!("{text} is not a number"))
                .with_line(self.number)
        })
    }

    fn usize(&self, index: usize) -> Result<usize, ScenarioError> {
        let text = self.field(index)?;
        text.parse::<usize>().map_err(|_| {
            ScenarioError::new(ScenarioErrorKind::BadArgument)
                .with_detail(format!("{text} is not an index"))
                .with_line(self.number)
        })
    }

    fn u64(&self, index: usize) -> Result<u64, ScenarioError> {
        let text = self.field(index)?;
        text.parse::<u64>().map_err(|_| {
            ScenarioError::new(ScenarioErrorKind::BadArgument)
                .with_detail(format!("{text} is not a count"))
                .with_line(self.number)
        })
    }

    fn boolean(&self, index: usize) -> Result<bool, ScenarioError> {
        let text = self.field(index)?;
        match text {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(ScenarioError::new(ScenarioErrorKind::BadArgument)
                .with_detail(format!("{text} is not a boolean"))
                .with_line(self.number)),
        }
    }

    /// Reports an error at this line.
    ///
    /// 在本行报告一个错误。
    fn fail(&self, kind: ScenarioErrorKind) -> ScenarioError {
        ScenarioError::new(kind).with_line(self.number)
    }
}

/// Splits text into significant lines, dropping blanks and comments.
///
/// 把文本拆分为有意义的行，丢弃空行与注释。
fn collect_lines(text: &str) -> Vec<Line<'_>> {
    text.lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let fields: Vec<&str> = trimmed.split_whitespace().collect();
            if fields.is_empty() {
                return None;
            }
            Some(Line {
                number: index + 1,
                fields,
            })
        })
        .collect()
}

/// The header keywords, which a resource or task pass must not see.
///
/// 头部关键字，资源或任务那一遍不得看到它们。
fn is_header(keyword: &str) -> bool {
    matches!(keyword, "scenario" | "epoch" | "time_limit" | "event_limit")
}

/// The resource pass.
///
/// 资源那一遍。
fn parse_resources(lines: &[Line<'_>]) -> Result<Scenario, ScenarioError> {
    let mut name: Option<String> = None;
    let mut scenario: Option<Scenario> = None;
    let mut saw_epoch = false;
    let mut saw_time_limit = false;
    let mut saw_event_limit = false;

    for line in lines {
        let keyword = line.keyword();
        // A resource must follow the header, because the scenario is built
        // when the name is read.
        // 资源必须跟在头部之后，因为场景是在读取名称时构建的。
        if keyword == "scenario" {
            if name.is_some() {
                return Err(line.fail(ScenarioErrorKind::DuplicateField));
            }
            if line.fields.len() < 2 {
                return Err(line.fail(ScenarioErrorKind::EmptyName));
            }
            let value = line.rest(1);
            name = Some(value.clone());
            scenario = Some(Scenario::new(value));
            continue;
        }

        let Some(current) = scenario.as_mut() else {
            return Err(line.fail(ScenarioErrorKind::MissingHeader));
        };

        match keyword {
            "epoch" => {
                if saw_epoch {
                    return Err(line.fail(ScenarioErrorKind::DuplicateField));
                }
                saw_epoch = true;
                let epoch = line.fuse(1)?;
                current
                    .set_epoch(epoch)
                    .map_err(|error| error.with_line(line.number))?;
            }
            "time_limit" => {
                if saw_time_limit {
                    return Err(line.fail(ScenarioErrorKind::DuplicateField));
                }
                saw_time_limit = true;
                let limit = line.fuse(1)?;
                current
                    .set_time_limit(limit)
                    .map_err(|error| error.with_line(line.number))?;
            }
            "event_limit" => {
                if saw_event_limit {
                    return Err(line.fail(ScenarioErrorKind::DuplicateField));
                }
                saw_event_limit = true;
                let limit = line.u64(1)?;
                if limit == 0 {
                    return Err(line.fail(ScenarioErrorKind::BadArgument));
                }
                *current = current.clone().with_event_limit(limit);
            }
            "resource" => {
                let id = ResourceId::new(line.usize(1)?);
                let capacity = line.fuse(3)?;
                let resource = Resource::new(id, capacity);
                current
                    .add_resource(resource)
                    .map_err(|error| error.with_line(line.number))?;
            }
            "window" => {
                let id = ResourceId::new(line.usize(1)?);
                let start = line.fuse(2)?;
                let end = line.fuse(3)?;
                let window = Window::new(start, end);
                let resource = current
                    .resources_mut()
                    .iter_mut()
                    .find(|resource| resource.id == id)
                    .ok_or_else(|| {
                        ScenarioError::new(ScenarioErrorKind::UnknownResource)
                            .with_detail(id.to_string())
                            .with_line(line.number)
                    })?;
                resource.add_window(window).map_err(|error| {
                    ScenarioError::new(ScenarioErrorKind::InvalidResource)
                        .with_detail(error.to_string())
                        .with_line(line.number)
                })?;
            }
            _ if is_header(keyword) => {
                return Err(line.fail(ScenarioErrorKind::MissingHeader));
            }
            // The task and precedence keywords are read in a later pass.
            // 任务与先后约束关键字在稍后那一遍中读取。
            "task" | "task_window" | "task_priority" | "task_mandatory" | "require"
            | "precedence" => {}
            _ => return Err(line.fail(ScenarioErrorKind::UnknownKeyword)),
        }
    }

    scenario.ok_or_else(|| ScenarioError::new(ScenarioErrorKind::MissingHeader))
}

/// The task pass.
///
/// 任务那一遍。
fn parse_tasks(lines: &[Line<'_>], scenario: &mut Scenario) -> Result<(), ScenarioError> {
    let mut current: Option<Task> = None;

    for line in lines {
        match line.keyword() {
            "task" => {
                flush(&mut current, scenario)?;
                let id = TaskId::new(line.usize(1)?);
                let duration = line.fuse(3)?;
                current = Some(Task::new(id, duration));
            }
            "task_window" => {
                let task = require_current(&current, line)?;
                let start = line.fuse(2)?;
                let end = line.fuse(3)?;
                current = Some(task.clone().with_window(start, end));
            }
            "task_priority" => {
                let task = require_current(&current, line)?;
                let priority = line.fuse(2)?;
                current = Some(task.clone().with_priority(priority));
            }
            "task_mandatory" => {
                let task = require_current(&current, line)?;
                let mandatory = line.boolean(2)?;
                current = Some(task.clone().with_mandatory(mandatory));
            }
            "require" => {
                let task = require_current(&current, line)?;
                let resource = ResourceId::new(line.usize(2)?);
                let demand = line.fuse(3)?;
                let updated = task
                    .clone()
                    .try_require(resource, demand)
                    .map_err(|error| {
                        ScenarioError::new(ScenarioErrorKind::BadArgument)
                            .with_detail(error.to_string())
                            .with_line(line.number)
                    })?;
                current = Some(updated);
            }
            _ => {}
        }
    }
    flush(&mut current, scenario)
}

/// Adds the task being built, if there is one.
///
/// 添加正在构建的任务（若有）。
fn flush(current: &mut Option<Task>, scenario: &mut Scenario) -> Result<(), ScenarioError> {
    if let Some(task) = current.take() {
        scenario.add_task(task)?;
    }
    Ok(())
}

/// Returns the task a qualifier line applies to, or an error naming the line.
///
/// 返回某限定行所适用的任务，或一个指明该行的错误。
fn require_current<'a>(
    current: &'a Option<Task>,
    line: &Line<'_>,
) -> Result<&'a Task, ScenarioError> {
    current.as_ref().ok_or_else(|| {
        ScenarioError::new(ScenarioErrorKind::MissingArgument)
            .with_detail(format!("{} before any task", line.keyword()))
            .with_line(line.number)
    })
}

/// The precedence pass.
///
/// 先后约束那一遍。
fn parse_precedence(lines: &[Line<'_>], scenario: &mut Scenario) -> Result<(), ScenarioError> {
    for line in lines {
        if line.keyword() != "precedence" {
            continue;
        }
        let before = TaskId::new(line.usize(1)?);
        let after = TaskId::new(line.usize(2)?);
        scenario
            .add_precedence(before, after)
            .map_err(|error| error.with_line(line.number))?;
    }
    Ok(())
}
