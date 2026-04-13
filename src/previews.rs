use crate::common::display_text;
use crate::raindrops::{ResolvedUpdateManySelection, UpdateManyArgs};
use crate::tags::{DeleteTagArgs, RenameTagArgs};
use puddle::models::collections::{Collection, CreateCollection, UpdateCollection};
use puddle::models::common::CollectionScope;
use puddle::models::raindrops::{CreateRaindrop, Raindrop, UpdateRaindrop};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

pub(crate) struct CreateRaindropPreview<'a> {
    payload: &'a CreateRaindrop,
}

impl<'a> CreateRaindropPreview<'a> {
    pub(crate) fn new(payload: &'a CreateRaindrop) -> Self {
        Self { payload }
    }
}

impl Display for CreateRaindropPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let link = &self.payload.link;

        writeln!(f, "Dry Run: Create Raindrop")?;
        write!(f, "Link: {link}")?;

        if let Some(title) = self.payload.title.as_deref() {
            write!(f, "\nTitle: {title}")?;
        }

        if let Some(collection) = self.payload.collection.as_ref() {
            let collection_scope = CollectionScope::from(collection.id);

            write!(f, "\nCollection: {collection_scope}")?;
        }

        if !self.payload.tags.is_empty() {
            let tags = format_tags(&self.payload.tags);

            write!(f, "\nTags: {tags}")?;
        }

        Ok(())
    }
}

pub(crate) struct UpdateRaindropPreview<'a> {
    existing_raindrop: &'a Raindrop,
    payload: &'a UpdateRaindrop,
}

impl<'a> UpdateRaindropPreview<'a> {
    pub(crate) fn new(existing_raindrop: &'a Raindrop, payload: &'a UpdateRaindrop) -> Self {
        Self {
            existing_raindrop,
            payload,
        }
    }
}

impl Display for UpdateRaindropPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let raindrop_id = self.existing_raindrop.id;
        let existing_title = display_text(self.existing_raindrop.title.as_deref(), "(untitled)");

        writeln!(f, "Dry Run: Update Raindrop")?;
        writeln!(f, "Target")?;
        writeln!(f, "ID: {raindrop_id}")?;
        writeln!(f, "Title: {existing_title}")?;

        let changes = self.existing_raindrop.change_lines(self.payload);

        if !changes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Changes")?;
            write!(f, "{}", changes.join("\n"))?;
        } else {
            writeln!(f)?;
            write!(f, "No changes found.")?;
        }

        Ok(())
    }
}

pub(crate) struct DeleteRaindropPreview<'a> {
    existing_raindrop: &'a Raindrop,
}

impl<'a> DeleteRaindropPreview<'a> {
    pub(crate) fn new(existing_raindrop: &'a Raindrop) -> Self {
        Self { existing_raindrop }
    }
}

impl Display for DeleteRaindropPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let raindrop_id = self.existing_raindrop.id;
        let existing_title = display_text(self.existing_raindrop.title.as_deref(), "(untitled)");

        writeln!(f, "Dry Run: Delete Raindrop")?;
        writeln!(f, "Target")?;
        write!(f, "ID: {raindrop_id}\nTitle: {existing_title}")
    }
}

pub(crate) struct UploadRaindropFilePreview<'a> {
    path: &'a PathBuf,
}

impl<'a> UploadRaindropFilePreview<'a> {
    pub(crate) fn new(path: &'a PathBuf) -> Self {
        Self { path }
    }
}

impl Display for UploadRaindropFilePreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let path = self.path.display();

        write!(f, "Dry Run: Upload Raindrop File\nPath: {path}")
    }
}

pub(crate) struct UploadRaindropCoverPreview<'a> {
    existing_raindrop: &'a Raindrop,
    path: &'a PathBuf,
}

impl<'a> UploadRaindropCoverPreview<'a> {
    pub(crate) fn new(existing_raindrop: &'a Raindrop, path: &'a PathBuf) -> Self {
        Self {
            existing_raindrop,
            path,
        }
    }
}

impl Display for UploadRaindropCoverPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let raindrop_id = self.existing_raindrop.id;
        let existing_title = display_text(self.existing_raindrop.title.as_deref(), "(untitled)");
        let path = self.path.display();

        writeln!(f, "Dry Run: Upload Raindrop Cover")?;
        writeln!(f, "Target")?;
        write!(
            f,
            "ID: {raindrop_id}\nTitle: {existing_title}\nPath: {path}"
        )
    }
}

pub(crate) struct CreateManyRaindropsPreview<'a> {
    input: &'a PathBuf,
    payloads: &'a [CreateRaindrop],
}

impl<'a> CreateManyRaindropsPreview<'a> {
    pub(crate) fn new(input: &'a PathBuf, payloads: &'a [CreateRaindrop]) -> Self {
        Self { input, payloads }
    }
}

impl Display for CreateManyRaindropsPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let input = self.input.display();
        let count = self.payloads.len();

        writeln!(f, "Dry Run: Create Many Raindrops")?;
        writeln!(f, "Input: {input}")?;
        writeln!(f, "Count: {count}")?;
        writeln!(f)?;
        writeln!(f, "Items to Create")?;

        for (index, payload) in self.payloads.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
                writeln!(f)?;
            }

            writeln!(
                f,
                "Title: {}",
                display_text(payload.title.as_deref(), "(untitled)")
            )?;
            write!(f, "Link: {}", payload.link)?;
        }

        Ok(())
    }
}

pub(crate) struct UpdateManyRaindropsPreview<'a> {
    selection: &'a ResolvedUpdateManySelection,
    args: &'a UpdateManyArgs,
}

impl<'a> UpdateManyRaindropsPreview<'a> {
    pub(crate) fn new(
        selection: &'a ResolvedUpdateManySelection,
        args: &'a UpdateManyArgs,
    ) -> Self {
        Self { selection, args }
    }
}

impl Display for UpdateManyRaindropsPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let matched_count = self.selection.targets.len();

        writeln!(f, "Dry Run: Update Many Raindrops")?;
        writeln!(f, "Matched: {matched_count}")?;

        let has_scope = !self.selection.from_collections.is_empty()
            || !self.selection.excluded_collections.is_empty()
            || self.selection.search.is_some();

        if has_scope {
            writeln!(f)?;
            writeln!(f, "Scope")?;
        }

        if !self.selection.from_collections.is_empty() {
            let from_collections = format_collection_scopes(&self.selection.from_collections);

            writeln!(f, "From Collections: {from_collections}")?;
        }

        if !self.selection.excluded_collections.is_empty() {
            let excluded_collections =
                format_collection_scopes(&self.selection.excluded_collections);

            writeln!(f, "Excluded Collections: {excluded_collections}")?;
        }

        if let Some(search) = self.selection.search.as_deref() {
            writeln!(f, "Search: {search}")?;
        }

        writeln!(f)?;
        writeln!(f, "Matched Targets")?;

        for target in &self.selection.targets {
            let target_title = display_text(target.title.as_deref(), "(untitled)");

            writeln!(f, "- #{} {}", target.id, target_title)?;
        }

        if self.args.to_collection.is_some() || !self.args.tags.is_empty() {
            let payload = UpdateRaindrop {
                title: None,
                excerpt: None,
                collection: self
                    .args
                    .to_collection
                    .map(|scope| scope.try_into())
                    .transpose()
                    .map_err(|_| fmt::Error)?,
                tags: (!self.args.tags.is_empty()).then_some(self.args.tags.clone()),
                extra: Default::default(),
            };

            writeln!(f)?;
            writeln!(f, "Changes")?;

            for (index, target) in self.selection.targets.iter().enumerate() {
                if index > 0 {
                    writeln!(f)?;
                }

                let target_id = target.id;
                let target_title = display_text(target.title.as_deref(), "(untitled)");

                writeln!(f, "ID: {target_id}")?;
                writeln!(f, "Title: {target_title}")?;

                for change in target.change_lines(&payload) {
                    writeln!(f, "{change}")?;
                }
            }
        }

        Ok(())
    }
}

pub(crate) struct DeleteManyRaindropsPreview<'a> {
    collection_scope: CollectionScope,
    targets: &'a [Raindrop],
}

impl<'a> DeleteManyRaindropsPreview<'a> {
    pub(crate) fn new(collection_scope: CollectionScope, targets: &'a [Raindrop]) -> Self {
        Self {
            collection_scope,
            targets,
        }
    }
}

impl Display for DeleteManyRaindropsPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let matched_count = self.targets.len();
        let collection_scope = self.collection_scope;

        writeln!(f, "Dry Run: Delete Many Raindrops")?;
        writeln!(f, "Matched: {matched_count}")?;
        writeln!(f)?;
        writeln!(f, "Scope")?;
        writeln!(f, "Collection: {collection_scope}")?;
        writeln!(f)?;
        writeln!(f, "Matched Targets")?;
        let targets = self
            .targets
            .iter()
            .map(|target| {
                let target_id = target.id;
                let target_title = display_text(target.title.as_deref(), "(untitled)");

                format!("ID: {target_id}\nTitle: {target_title}")
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        write!(f, "{targets}")?;

        Ok(())
    }
}

pub(crate) struct CreateCollectionPreview<'a> {
    payload: &'a CreateCollection,
}

impl<'a> CreateCollectionPreview<'a> {
    pub(crate) fn new(payload: &'a CreateCollection) -> Self {
        Self { payload }
    }
}

impl Display for CreateCollectionPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let title = &self.payload.title;

        writeln!(f, "Dry Run: Create Collection")?;
        write!(f, "Title: {title}")?;

        if let Some(parent) = self.payload.parent {
            write!(f, "\nParent: {parent}")?;
        }

        Ok(())
    }
}

pub(crate) struct UpdateCollectionPreview<'a> {
    existing_collection: &'a Collection,
    payload: &'a UpdateCollection,
}

impl<'a> UpdateCollectionPreview<'a> {
    pub(crate) fn new(existing_collection: &'a Collection, payload: &'a UpdateCollection) -> Self {
        Self {
            existing_collection,
            payload,
        }
    }
}

impl Display for UpdateCollectionPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let collection_id = self.existing_collection.id;
        let existing_title = display_text(self.existing_collection.title.as_deref(), "(untitled)");

        writeln!(f, "Dry Run: Update Collection")?;
        writeln!(f, "Target")?;
        writeln!(f, "ID: {collection_id}")?;
        writeln!(f, "Title: {existing_title}")?;

        let mut changes = Vec::new();

        if let Some(title) = self.payload.title.as_deref() {
            changes.push(format!("Title: {existing_title} -> {title}"));
        }

        if let Some(parent) = self.payload.parent {
            let existing_parent = display_parent(self.existing_collection.parent);

            changes.push(format!("Parent: {existing_parent} -> {parent}"));
        }

        if !changes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Changes")?;
            write!(f, "{}", changes.join("\n"))?;
        }

        Ok(())
    }
}

pub(crate) struct DeleteCollectionPreview<'a> {
    existing_collection: &'a Collection,
}

impl<'a> DeleteCollectionPreview<'a> {
    pub(crate) fn new(existing_collection: &'a Collection) -> Self {
        Self {
            existing_collection,
        }
    }
}

impl Display for DeleteCollectionPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let collection_id = self.existing_collection.id;
        let existing_title = display_text(self.existing_collection.title.as_deref(), "(untitled)");

        writeln!(f, "Dry Run: Delete Collection")?;
        writeln!(f, "Target")?;
        writeln!(f, "ID: {collection_id}")?;
        write!(f, "Title: {existing_title}")
    }
}

pub(crate) struct UploadCollectionCoverPreview<'a> {
    existing_collection: &'a Collection,
    path: &'a PathBuf,
}

impl<'a> UploadCollectionCoverPreview<'a> {
    pub(crate) fn new(existing_collection: &'a Collection, path: &'a PathBuf) -> Self {
        Self {
            existing_collection,
            path,
        }
    }
}

impl Display for UploadCollectionCoverPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let collection_id = self.existing_collection.id;
        let existing_title = display_text(self.existing_collection.title.as_deref(), "(untitled)");
        let path = self.path.display();

        writeln!(f, "Dry Run: Upload Collection Cover")?;
        writeln!(f, "Target")?;
        writeln!(f, "ID: {collection_id}")?;
        writeln!(f, "Title: {existing_title}")?;
        write!(f, "Path: {path}")
    }
}

pub(crate) struct RenameTagPreview<'a> {
    args: &'a RenameTagArgs,
}

impl<'a> RenameTagPreview<'a> {
    pub(crate) fn new(args: &'a RenameTagArgs) -> Self {
        Self { args }
    }
}

impl Display for RenameTagPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let collection_id = self.args.collection_id;
        let from_tag = &self.args.find;
        let to_tag = &self.args.replace;

        writeln!(f, "Dry Run: Rename Tag")?;
        writeln!(f, "Target")?;
        writeln!(f, "Collection: {collection_id}")?;
        writeln!(f, "Tag: {from_tag}")?;
        writeln!(f)?;
        writeln!(f, "Changes")?;
        write!(f, "Tag: {from_tag} -> {to_tag}")
    }
}

pub(crate) struct DeleteTagPreview<'a> {
    args: &'a DeleteTagArgs,
}

impl<'a> DeleteTagPreview<'a> {
    pub(crate) fn new(args: &'a DeleteTagArgs) -> Self {
        Self { args }
    }
}

impl Display for DeleteTagPreview<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let collection_id = self.args.collection_id;
        let tag = &self.args.tag;

        writeln!(f, "Dry Run: Delete Tag")?;
        writeln!(f, "Target")?;
        writeln!(f, "Collection: {collection_id}")?;
        write!(f, "Tag: {tag}")
    }
}

fn format_collection_scopes(scopes: &[CollectionScope]) -> String {
    scopes
        .iter()
        .map(|scope| scope.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "(none)".to_string()
    } else {
        tags.join(", ")
    }
}

fn display_parent(parent: Option<i64>) -> String {
    parent
        .map(|value| value.to_string())
        .unwrap_or_else(|| "(none)".to_string())
}
