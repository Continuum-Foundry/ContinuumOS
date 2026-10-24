//! Ephemeral hosted L0/L1 generations. No native code or guest update transport.
use crate::{Result, custom, hash, root, write_json};
use custom_ui::{
    Rect,
    input::Controller,
    layout::Plan,
    paint,
    scene::{Scene, Tokens, Tree},
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Read,
    path::Path,
    sync::{Arc, Weak},
    time::Instant,
};

pub const MAX_FILE: usize = 128 * 1024;
const GENERATION_RESERVE: usize = 8 * 1024 * 1024;
const MAX_GENERATIONS: usize = 3;
pub const DEBOUNCE_US: u64 = 100_000;
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn checked<T>(v: custom_ui::Result<T>) -> Result<T> {
    v.map_err(|e| format!("resource: {e:?}"))
}
fn keys(v: &Value, expected: &[&str]) -> Result<()> {
    let o = v.as_object().ok_or("Expected resource object")?;
    need(
        o.len() == expected.len() && o.keys().all(|k| expected.contains(&k.as_str())),
        "Unknown or missing resource field",
    )
}
fn number(v: &Value) -> Result<u64> {
    v.as_u64().ok_or("Expected resource integer".into())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let f = fs::File::open(path).map_err(|e| format!("Resource file: {e}"))?;
    let mut bytes = Vec::new();
    f.take(MAX_FILE as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() <= MAX_FILE, "Resource file exceeds 128 KiB")?;
    Ok(bytes)
}
fn digest(v: &Value) -> String {
    hash(&serde_json::to_vec(v).unwrap())
}
fn assets() -> Result<Value> {
    let lock: Value = serde_json::from_slice(
        &fs::read(root().join("fixtures/m2/text-v1/fonts.lock.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(json!(
        lock["fonts"]
            .as_array()
            .ok_or("Missing font lock")?
            .iter()
            .map(|f| json!({"name":f["name"],"sha256":f["sha256"],"bytes":f["bytes"]}))
            .collect::<Vec<_>>()
    ))
}
fn fixture() -> Result<Value> {
    let mut c = crate::ui_fixtures::checked_inputs()?;
    // Editable resources start with the actual selected 1280x720 baseline,
    // rather than reverting the live view to the historical 640x360 corpus.
    let source = custom::tree(&c["layout"])?;
    let plan = crate::selected::reference_plan(&source, 1280.0, 720.0)?;
    let (arranged, _) = checked(plan.arrange(&source, Rect([0.0, 0.0, 1280.0, 720.0])))?;
    for n in c.get_mut("layout").ok_or("Template layout")?["nodes"]
        .as_array_mut()
        .ok_or("Template nodes")?
    {
        n["rect"] = json!(
            checked(arranged.node(n["id"].as_str().ok_or("Template ID")?))?
                .rect
                .0
        );
    }
    Ok(
        json!({"schema_version":1,"build_id":"initial","generation":7,"base_generation":6,"tokens":c["tokens"],"variant":c["variants"]["variants"][0],"layout":c["layout"],"assets":assets()?}),
    )
}
pub fn template(path: &Path) -> Result<()> {
    write_json(path, &fixture()?)
}

/// Validate a source edit and atomically replace the completed build file. A
/// failed build leaves the prior output bytes untouched. No dependency fetching.
pub fn build(input: &Path, output: &Path) -> Result<()> {
    let start = Instant::now();
    let bytes = read(input)?;
    let v: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("Resource builder JSON: {e}"))?;
    let generation = Resources::parse(v, &assets()?)?;
    let _fonts = crate::text::inputs()?;
    let completed = serde_json::to_vec(&generation.payload).map_err(|e| e.to_string())?;
    need(
        completed.len() <= MAX_FILE,
        "Completed resource build exceeds 128 KiB",
    )?;
    let temporary = output.with_extension(format!("build-{}.tmp", std::process::id()));
    fs::write(&temporary, &completed).map_err(|e| format!("Resource build output: {e}"))?;
    fs::rename(&temporary, output).map_err(|e| format!("Resource build publication: {e}"))?;
    println!(
        "{}",
        json!({"outcome":"built","build_id":generation.payload["build_id"],"generation":generation.id,"digest":generation.digest,"source_sha256":hash(&bytes),"completed_sha256":hash(&completed),"build_ns":start.elapsed().as_nanos(),"scope":"ephemeral-hosted"})
    );
    Ok(())
}

struct Generation {
    id: u64,
    digest: String,
    source: Tree,
    tokens: Tokens,
    payload: Value,
}
/// Reader handles cannot be forged and own all data needed by their generation.
/// The registry retains only weak references; MAX_GENERATIONS bounds live roots.
pub struct Reader(Arc<Generation>);
impl Reader {
    pub fn id(&self) -> u64 {
        self.0.id
    }
    pub fn digest(&self) -> &str {
        &self.0.digest
    }
}
pub struct Resources {
    active: Arc<Generation>,
    retired: Vec<Weak<Generation>>,
    pinned: Value,
    clock: Instant,
    request: u64,
    pub trace: Vec<Value>,
    pub status: Value,
    reserve: usize,
}
impl Resources {
    pub fn new() -> Result<Self> {
        let v = fixture()?;
        let pinned = v["assets"].clone();
        let generation = Self::parse(v, &pinned)?;
        Ok(Self {
            active: Arc::new(generation),
            retired: Vec::new(),
            pinned,
            clock: Instant::now(),
            request: 0,
            trace: Vec::new(),
            status: json!({"outcome":"initial","active_generation":7,"scope":"ephemeral-hosted"}),
            reserve: MAX_GENERATIONS * GENERATION_RESERVE,
        })
    }
    pub fn reader(&self) -> Reader {
        Reader(self.active.clone())
    }
    pub fn id(&self) -> u64 {
        self.active.id
    }
    pub fn payload(&self) -> Value {
        self.active.payload.clone()
    }
    pub fn source(&self) -> &Tree {
        &self.active.source
    }
    pub fn now(&self) -> u64 {
        self.clock.elapsed().as_micros() as u64
    }
    pub fn live(&mut self) -> usize {
        self.retired.retain(|w| w.strong_count() != 0);
        1 + self.retired.len()
    }
    pub fn query(&mut self) -> Value {
        let live = self.live();
        json!({"active_generation":self.id(),"digest":self.active.digest,"live_generations":live,"reserved_generation_bytes":live*GENERATION_RESERVE,"generation_budget_bytes":self.reserve,"shared_font_bytes":self.pinned.as_array().unwrap().iter().map(|v|v["bytes"].as_u64().unwrap()).sum::<u64>(),"status":self.status})
    }
    fn parse(v: Value, pinned: &Value) -> Result<Generation> {
        keys(
            &v,
            &[
                "schema_version",
                "build_id",
                "generation",
                "base_generation",
                "tokens",
                "variant",
                "layout",
                "assets",
            ],
        )?;
        need(v["schema_version"] == 1, "Unsupported resource version")?;
        need(
            v["build_id"]
                .as_str()
                .is_some_and(|s| !s.is_empty() && s.len() <= 64),
            "Invalid build ID",
        )?;
        let id = number(&v["generation"])?;
        need(
            number(&v["base_generation"])?.checked_add(1) == Some(id),
            "Nonconsecutive resource generation",
        )?;
        let t = &v["tokens"];
        keys(
            t,
            &[
                "schema_version",
                "generation",
                "colors",
                "fonts",
                "font_sizes",
                "spacing",
                "radii",
                "effects",
                "motion",
            ],
        )?;
        need(
            t["schema_version"] == 1 && t["generation"] == id,
            "Token version/generation mismatch",
        )?;
        let variant = &v["variant"];
        keys(
            variant,
            &["id", "colors", "reduced_effects", "reduced_motion"],
        )?;
        need(variant["id"] == "standard", "Unsupported variant ID")?;
        let colors = [
            "surface", "panel", "text", "muted", "accent", "focus", "danger",
        ];
        keys(&t["colors"], &colors)?;
        keys(&variant["colors"], &colors)?;
        need(
            t["colors"] == variant["colors"],
            "Conflicting token palette",
        )?;
        keys(&t["fonts"], &["body", "editor", "fallback"])?;
        need(
            t["fonts"]
                == json!({"body":"ui-sans","editor":"ui-mono","fallback":"complex-fallback"}),
            "Missing or unsupported font binding",
        )?;
        keys(&t["font_sizes"], &["body", "editor", "title"])?;
        for k in ["body", "editor", "title"] {
            need(
                (8..=48).contains(&number(&t["font_sizes"][k])?),
                "Font size bound",
            )?;
        }
        keys(&t["spacing"], &["small", "medium", "large"])?;
        for k in ["small", "medium", "large"] {
            need(
                t["spacing"][k]
                    .as_f64()
                    .is_some_and(|x| x.is_finite() && (0.0..=64.0).contains(&x)),
                "Spacing bound",
            )?;
        }
        keys(&t["radii"], &["panel", "control"])?;
        keys(
            &t["effects"],
            &["kind", "radius", "passes", "tint", "fallback"],
        )?;
        need(
            t["effects"]["kind"] == "backdrop-box-blur"
                && t["effects"]["radius"] == 8
                && t["effects"]["passes"] == 3,
            "Unsupported effect",
        )?;
        keys(&t["motion"], &["short_ms", "long_ms", "reduced_ms"])?;
        need(
            t["motion"] == json!({"short_ms":120,"long_ms":240,"reduced_ms":0}),
            "Unsupported motion timings",
        )?;
        need(
            v["assets"] == *pinned,
            "Missing, corrupt or unsupported font/image asset",
        )?;
        let l = &v["layout"];
        keys(
            l,
            &[
                "schema_version",
                "generation",
                "base_revision",
                "new_revision",
                "root",
                "nodes",
            ],
        )?;
        need(
            l["schema_version"] == 1 && l["generation"] == id,
            "Layout version/generation mismatch",
        )?;
        need(
            number(&l["base_revision"])?.checked_add(1) == Some(number(&l["new_revision"])?),
            "Layout revision mismatch",
        )?;
        let nodes = l["nodes"].as_array().ok_or("Expected layout nodes")?;
        need(
            !nodes.is_empty() && nodes.len() <= 1024,
            "Layout complexity",
        )?;
        for n in nodes {
            keys(
                n,
                &[
                    "id",
                    "kind",
                    "children",
                    "rect",
                    "clip",
                    "opacity",
                    "text",
                    "action",
                    "resource",
                    "semantic",
                    "focusable",
                ],
            )?;
            keys(&n["semantic"], &["id", "role", "name"])?;
            // The current painter has these concrete bindings. Reject incompatible
            // roles/actions instead of silently turning an edit into another action.
            let (action, role) = match n["kind"].as_str().ok_or("Node kind")? {
                "editor" => ("edit.insert@1", "textbox"),
                "text-field" => ("search.set@1", "textbox"),
                "virtual-list" => ("files.select@1", "list"),
                "button" => ("launcher.open@1", "button"),
                "toggle" => {
                    let a = n["action"].as_str().ok_or("Toggle action")?;
                    need(
                        ["settings.effects@1", "settings.motion@1"].contains(&a),
                        "Unsupported toggle action",
                    )?;
                    (a, "checkbox")
                }
                _ => ("", "group"),
            };
            // Labels may use the locked text role; groups have no action.
            need(
                n["action"] == action
                    && (n["semantic"]["role"] == role
                        || (n["kind"] == "label" && n["semantic"]["role"] == "text")),
                "Incompatible action/semantic binding",
            )?;
            let bound_id = match n["kind"].as_str().unwrap() {
                "editor" => Some("editor"),
                "text-field" => Some("search"),
                "virtual-list" => Some("files-window"),
                "button" => Some("launch-editor"),
                "toggle" => Some(if action == "settings.effects@1" {
                    "reduced-effects"
                } else {
                    "reduced-motion"
                }),
                _ => None,
            };
            need(
                bound_id.is_none_or(|id| n["id"] == id),
                "Unsupported control/action ID binding",
            )?;
            need(
                n["focusable"].as_bool() == Some(bound_id.is_some()),
                "Incompatible control focus binding",
            )?;
            need(n["resource"] != "icon.missing", "Missing image asset")?;
            for text in [&n["text"], &n["semantic"]["name"]] {
                text_core::validate_text(text.as_str().ok_or("Expected text")?)
                    .map_err(|e| format!("Resource text: {e:?}"))?;
            }
        }
        let source = custom::tree(l)?;
        // These stable shell bindings remain required in this bounded L1 subset.
        need(source.root == "desktop", "Unsupported shell root")?;
        for (id, kind) in [
            ("desktop", custom_ui::scene::Kind::Overlay),
            ("editor-window", custom_ui::scene::Kind::Column),
            ("files-window", custom_ui::scene::Kind::List),
        ] {
            need(
                checked(source.node(id))?.kind == kind,
                "Required shell binding missing or replaced",
            )?;
        }
        need(
            source.nodes.iter().any(|n| n.focusable),
            "No eligible focus target",
        )?;
        let tokens = custom::tokens(t, variant)?;
        Ok(Generation {
            id,
            digest: digest(&v),
            source,
            tokens,
            payload: v,
        })
    }
    /// Called on the serialized host event loop. Drain accepted input against its
    /// old geometry first. Candidate reconciliation is tested on a bounded copy;
    /// the authoritative provider is never replaced or checkpoint-restored.
    pub fn apply(
        &mut self,
        path: &Path,
        scene: &mut Scene,
        c: &mut Controller,
        engine: &mut text_core::Engine<'_>,
        width: u32,
        height: u32,
        scale: u16,
    ) -> Result<()> {
        self.request += 1;
        let mut row = json!({"request_id":self.request,"receive_us":self.now(),"active_before":self.id(),"scope":"ephemeral-hosted"});
        let outcome: Result<()> = (|| {
            need(self.trace.len() < 1024, "Resource request trace budget")?;
            checked(c.drain(&scene.tree))?;
            need(
                (self.live() + 1) * GENERATION_RESERVE <= self.reserve,
                "Resource staging headroom exhausted; release readers then retry",
            )?;
            need(
                path.as_os_str().len() <= 1024,
                "Resource path exceeds 1024 code units",
            )?;
            let bytes = read(path)?;
            row["source_sha256"] = json!(hash(&bytes));
            let v: Value = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Incomplete/invalid resource JSON: {e}"))?;
            row["build_id"] = json!(v["build_id"].as_str().filter(|s| s.len() <= 64));
            row["generation"] = json!(v["generation"].as_u64());
            need(
                v["base_generation"] == self.id(),
                "Stale base generation; query active generation",
            )?;
            let next = Self::parse(v, &self.pinned)?;
            // Assets are shared immutable startup bytes, already hash-checked by
            // text::inputs. No arbitrary file paths or new parsers enter a reload.
            row["digest"] = json!(next.digest);
            row["validated_us"] = json!(self.now());
            let view = Rect([
                0.0,
                0.0,
                width as f32 * 100.0 / scale as f32,
                height as f32 * 100.0 / scale as f32,
            ]);
            let plan = checked(Plan::anchored(&next.source))?;
            let (mut arranged, _) = checked(plan.arrange(&next.source, view))?;
            arranged.revision = scene
                .tree
                .revision
                .checked_add(1)
                .ok_or("Scene revision exhaustion")?;
            let mut stage = checked(Scene::new(arranged, next.tokens.clone()))?;
            stage.selected = true;
            let focus = &c.provider.model().session().focus;
            let incompatible_focus = scene.tree.node(focus).ok().is_some_and(|old| {
                stage.tree.node(focus).map_or(true, |new| {
                    old.kind != new.kind
                        || old.action != new.action
                        || old.semantic_id != new.semantic_id
                        || !new.focusable
                })
            });
            let mut trial = c.clone();
            if incompatible_focus {
                trial.reset("focus-owner-removed-or-replaced", 0);
            }
            checked(trial.reconcile(&stage.tree))?;
            stage.motion = scene.motion.clone();
            stage.motion.retain(&stage.tree);
            let reduced =
                stage.tokens.reduced_motion || trial.provider.model().preferences().reduced_motion;
            if reduced {
                stage.motion.settle();
            }
            checked(stage.motion.timers(
                trial.provider.model().session().focus == "editor",
                false,
                !trial.capture.is_empty(),
                scene.motion.now(),
                reduced,
            ))?;
            row["motion_policy"] =
                json!("retain compatible transitions; rearm caret/drag; reduced motion settles");
            // A visual affinity belongs to its shaped geometry, not just an ID.
            trial.visual_caret = None;
            let len = (width as usize)
                .checked_mul(height as usize)
                .and_then(|x| x.checked_mul(4))
                .filter(|n| *n <= 16 * 1024 * 1024)
                .ok_or("Staging surface exceeds 16 MiB")?;
            let mut pixels = vec![0; len];
            let mut canvas = checked(paint::Canvas::new(
                &mut pixels,
                width,
                height,
                width * 4,
                scale,
            ))?;
            checked(paint::render(
                &mut stage,
                trial.provider.model(),
                trial.composition.as_ref().map(|p| p.preedit()),
                engine,
                &mut canvas,
            ))?;
            let sem = checked(custom_ui::semantics::Snapshot::build(
                &stage.tree,
                trial.provider.model(),
            ))?;
            row["staged_us"] = json!(self.now());
            row["stage_rgba_sha256"] = json!(hash(&pixels));
            row["semantic_revision"] = json!(sem.scene_revision);
            row["stage_bytes"] = json!(len);
            // No callbacks/interleaving or fallible rendering after this point.
            if incompatible_focus {
                c.reset("focus-owner-removed-or-replaced", 0);
            }
            checked(c.reconcile(&stage.tree))?;
            c.visual_caret = None;
            row["cancellation_reason"] = json!(c.reset_reason);
            row["focus_after"] = json!(c.provider.model().session().focus);
            *scene = stage;
            let old = std::mem::replace(&mut self.active, Arc::new(next));
            if Arc::strong_count(&old) > 1 {
                self.retired.push(Arc::downgrade(&old));
            }
            row["published_us"] = json!(self.now());
            Ok(())
        })();
        row["outcome"] = json!(if outcome.is_ok() {
            "published"
        } else {
            "rejected"
        });
        if let Err(e) = &outcome {
            row["reason"] = json!(e.chars().take(1024).collect::<String>());
        }
        row["active_generation"] = json!(self.id());
        self.status = row.clone();
        if self.trace.len() < 1024 {
            self.trace.push(row);
        }
        outcome
    }
    pub fn first_frame(&mut self) -> Value {
        if self.status["outcome"] == "published" && self.status["first_frame_us"].is_null() {
            let now = self.now();
            self.status["first_frame_us"] = json!(now);
            if let Some(r) = self.trace.last_mut() {
                r["first_frame_us"] = json!(now);
            }
        }
        self.query()
    }
}

/// One pending content hash. Polls are bounded reads; repeated identical saves
/// restart neither debounce nor validation. Failed content is retried only on edit.
#[derive(Default)]
pub struct Trigger {
    observed: Option<String>,
    pending: Option<(String, u64)>,
    pub coalesced: u64,
}
impl Trigger {
    pub fn poll(&mut self, path: &Path, now: u64) -> Result<bool> {
        let bytes = match read(path) {
            Ok(v) => v,
            Err(e) => {
                self.pending = None;
                self.observed = None;
                return Err(e);
            }
        };
        let h = hash(&bytes);
        if self.observed.as_ref() != Some(&h) {
            if self.pending.is_some() {
                self.coalesced += 1;
            }
            self.observed = Some(h.clone());
            self.pending = Some((h, now));
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|(_, at)| now.saturating_sub(*at) >= DEBOUNCE_US)
        {
            self.pending = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests;

pub fn execute() -> Result<()> {
    tests_run::execute()
}
mod tests_run;
