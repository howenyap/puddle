use crate::app::CliApp;
use clap::{Args, Subcommand};
use puddle::models::filters::{FilterBucket, FilterCount, FiltersResponse};

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub(crate) enum FiltersCommand {
    #[command(about = "List filters for a collection")]
    List(ListFiltersArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub(crate) struct ListFiltersArgs {
    #[arg(allow_hyphen_values = true)]
    pub(crate) collection_id: i64,
}

impl CliApp {
    pub(crate) async fn run_filters(
        &self,
        command: FiltersCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            FiltersCommand::List(args) => self.filters_list(args).await,
        }
    }

    async fn filters_list(&self, args: ListFiltersArgs) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.filters().list(args.collection_id).await?;
        print_filters(&response.data);
        Ok(())
    }
}

fn print_filters(filters: &FiltersResponse) {
    let output = format_filters(filters);
    if !output.is_empty() {
        println!("{output}");
    }
}

fn format_filters(filters: &FiltersResponse) -> String {
    let mut groups = Vec::new();

    push_count(&mut groups, "total", filters.total.as_ref());
    push_count(&mut groups, "highlights", filters.highlights.as_ref());
    push_count(&mut groups, "notag", filters.notag.as_ref());
    push_count(&mut groups, "broken", filters.broken.as_ref());
    push_count(&mut groups, "duplicates", filters.duplicates.as_ref());
    push_count(&mut groups, "important", filters.important.as_ref());
    push_buckets(&mut groups, "tags", &filters.tags);
    push_buckets(&mut groups, "types", &filters.types);
    push_buckets(&mut groups, "created", &filters.created);

    groups.join("\n\n")
}

fn push_count(groups: &mut Vec<String>, label: &str, count: Option<&FilterCount>) {
    if let Some(count) = count {
        groups.push(format!("{label}: {}", count.count));
    }
}

fn push_buckets(groups: &mut Vec<String>, label: &str, buckets: &[FilterBucket]) {
    if buckets.is_empty() {
        return;
    }

    groups.push(
        std::iter::once(format!("{label}:"))
            .chain(
                buckets
                    .iter()
                    .map(|bucket| format!("  {} ({})", bucket.id, bucket.count)),
            )
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, Command};
    use clap::Parser;
    use std::collections::HashMap;

    #[test]
    fn parses_negative_collection_id_for_filters() {
        let cli = Cli::try_parse_from(["puddle", "filters", "list", "-1"]).unwrap();

        assert_eq!(
            Some(Command::Filters {
                command: FiltersCommand::List(ListFiltersArgs { collection_id: -1 }),
            }),
            cli.command
        );
    }

    #[test]
    fn formats_filter_groups_with_blank_lines_between_sections() {
        let filters = FiltersResponse {
            result: true,
            broken: None,
            duplicates: None,
            important: None,
            notag: Some(FilterCount {
                count: 39,
                extra: HashMap::new(),
            }),
            total: Some(FilterCount {
                count: 67,
                extra: HashMap::new(),
            }),
            highlights: Some(FilterCount {
                count: 3,
                extra: HashMap::new(),
            }),
            created: vec![FilterBucket {
                id: "2026-03".to_string(),
                count: 9,
                extra: HashMap::new(),
            }],
            tags: vec![FilterBucket {
                id: "Rust".to_string(),
                count: 4,
                extra: HashMap::new(),
            }],
            types: vec![FilterBucket {
                id: "article".to_string(),
                count: 30,
                extra: HashMap::new(),
            }],
            collection_id: Some(-1),
            extra: HashMap::new(),
        };

        let expected = [
            "total: 67",
            "highlights: 3",
            "notag: 39",
            "tags:\n  Rust (4)",
            "types:\n  article (30)",
            "created:\n  2026-03 (9)",
        ]
        .join("\n\n");

        let actual = format_filters(&filters);

        assert_eq!(expected, actual);
    }
}
