use jujutsu_lib::commit::Commit;
use jujutsu_lib::matchers::Matcher;
use jujutsu_lib::tree::{Tree, TreeDiffIterator};
use jujutsu_lib::{conflicts, diff, files, rewrite, tree};
/// Returns a list of requested diff formats, which will never be empty.
pub fn diff_formats_for(ui: &Ui, args: &DiffFormatArgs) -> Vec<DiffFormat> {
    let formats = diff_formats_from_args(args);
    if formats.is_empty() {
        vec![default_diff_format(ui)]
        formats
    }
}

/// Returns a list of requested diff formats for log-like commands, which may be
/// empty.
pub fn diff_formats_for_log(ui: &Ui, args: &DiffFormatArgs, patch: bool) -> Vec<DiffFormat> {
    let mut formats = diff_formats_from_args(args);
    // --patch implies default if no format other than --summary is specified
    if patch && matches!(formats.as_slice(), [] | [DiffFormat::Summary]) {
        formats.push(default_diff_format(ui));
        formats.dedup();
    }
    formats
}

fn diff_formats_from_args(args: &DiffFormatArgs) -> Vec<DiffFormat> {
    [
        (args.summary, DiffFormat::Summary),
        (args.git, DiffFormat::Git),
        (args.color_words, DiffFormat::ColorWords),
    ]
    .into_iter()
    .filter_map(|(arg, format)| arg.then(|| format))
    .collect()
}

fn default_diff_format(ui: &Ui) -> DiffFormat {
    match ui.settings().config().get_string("diff.format").as_deref() {
        Ok("summary") => DiffFormat::Summary,
        Ok("git") => DiffFormat::Git,
        Ok("color-words") => DiffFormat::ColorWords,
        _ => DiffFormat::ColorWords,
    from_tree: &Tree,
    to_tree: &Tree,
    matcher: &dyn Matcher,
    formats: &[DiffFormat],
    for format in formats {
        let tree_diff = from_tree.diff(to_tree, matcher);
        match format {
            DiffFormat::Summary => {
                show_diff_summary(formatter, workspace_command, tree_diff)?;
            }
            DiffFormat::Git => {
                show_git_diff(formatter, workspace_command, tree_diff)?;
            }
            DiffFormat::ColorWords => {
                show_color_words_diff(formatter, workspace_command, tree_diff)?;
            }
pub fn show_patch(
    formatter: &mut dyn Formatter,
    workspace_command: &WorkspaceCommandHelper,
    commit: &Commit,
    matcher: &dyn Matcher,
    formats: &[DiffFormat],
) -> Result<(), CommandError> {
    let parents = commit.parents();
    let from_tree = rewrite::merge_commit_trees(workspace_command.repo().as_repo_ref(), &parents);
    let to_tree = commit.tree();
    show_diff(
        formatter,
        workspace_command,
        &from_tree,
        &to_tree,
        matcher,
        formats,
    )
}

    from_tree: &Tree,
    to_tree: &Tree,
    matcher: &dyn Matcher,
    formats: &[DiffFormat],
    show_diff(
        &mut formatter,
        workspace_command,
        from_tree,
        to_tree,
        matcher,
        formats,
    )?;
                    formatter.write_str(&format!("Added {description} {ui_path}:\n"))
                    formatter.write_str(&format!("{description} {ui_path}:\n"))
                    formatter.write_str(&format!("Removed {description} {ui_path}:\n"))
                    writeln!(formatter, "diff --git a/{path_string} b/{path_string}")?;
                    writeln!(formatter, "+++ b/{path_string}")
                    writeln!(formatter, "diff --git a/{path_string} b/{path_string}")?;
                        writeln!(formatter, "--- a/{path_string}")?;
                        writeln!(formatter, "+++ b/{path_string}")?;
                    writeln!(formatter, "diff --git a/{path_string} b/{path_string}")?;
                    writeln!(formatter, "--- a/{path_string}")?;