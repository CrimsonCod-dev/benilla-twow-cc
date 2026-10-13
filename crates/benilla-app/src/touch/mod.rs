//! Touch input: one arbiter that decides, **once per finger**, what that finger is for.
//!
//! # Why this module exists
//!
//! Touch used to be bolted onto the mouse path inside [`crate::ui_script::input::feed_ui_input`]:
//! it picked "the" touch out of the [`Touches`] resource and fed its position in as if it were a
//! cursor. That shape produced the three symptoms reported on-device, and all three are the same
//! bug wearing different clothes:
//!
//! 1. **Taps needed spamming.** Bevy's [`Touches::iter`] yields only *currently pressed* touches.
//!    On the frame a finger lifts, the touch has already moved to the just-released set, so
//!    `iter()` no longer yields it. With no real mouse attached, `Window::cursor_position()` is
//!    `None` too — so the whole `if let Some(cursor)` block was **skipped on every release
//!    frame**, and the UI's release half never dispatched. A click that never releases is a click
//!    that never fires. You could only land one by accident, hence: tap, tap, tap.
//! 2. **A second resting finger "fixed" it.** With another finger down, `iter()` still yielded
//!    *something* on the release frame, so the block ran and the release got through — via the
//!    wrong finger's coordinates, but through. That is the entire "two-finger quirk".
//! 3. **Timing mattered.** A tap that pressed and released inside one frame lost both halves.
//!
//! The fix is not a patch to the selection expression — it is giving touch its own pass with a
//! **sticky role per finger**, which is what every shipping touch client does:
//!
//! - A finger is classified **once, on touchdown**, and keeps that role until it lifts.
//! - Roles never migrate. A finger that started on the joystick cannot become a look-drag halfway
//!   through, and a finger that started over a button cannot start turning the camera because it
//!   drifted off the button.
//! - Each role has exactly one consumer, so two fingers can never fight over one axis.
//!
//! # The roles
//!
//! | Role | Claimed when | Drives |
//! |---|---|---|
//! | [`Role::Joystick`] | touchdown inside the joystick zone (and no joystick finger yet) | [`TouchMove`] → WASD |
//! | [`Role::Ui`] | touchdown over a mouse-enabled UI frame | [`TouchPointer`] → the UI pointer |
//! | [`Role::Look`] | anything else | [`TouchLook`] → camera yaw/pitch, and world taps. One finger turns the body (right-click's job); two or more orbit the camera only (left-click's — `TouchLook::finger_count` is how `player::camera` tells them apart) |
//!
//! The order matters: the joystick is tested first so it keeps working even if an addon parks a
//! transparent full-screen frame over the world (a real failure mode — a frame that takes the
//! mouse would otherwise swallow the whole left thumb).
//!
//! # Mice are unaffected
//!
//! Nothing here reads or writes [`bevy::input::ButtonInput<MouseButton>`], and [`TouchPointer`] is
//! only consulted by the UI pass when the window reports no cursor of its own. A real mouse takes
//! the same path it always did.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;

mod joystick;

/// What a finger was claimed for on touchdown. Assigned once; never reassigned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Role {
    /// Left-thumb movement stick.
    Joystick,
    /// Driving the UI pointer (this finger came down over a mouse-enabled frame).
    Ui,
    /// Free-swipe camera look, and — if it barely moves before lifting — a world tap.
    Look,
}

/// One finger the arbiter is tracking.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Finger {
    pub id: u64,
    pub role: Role,
    /// Where it came down (logical px, y-down from top-left — the space `Touch::position()` and
    /// `Window::cursor_position()` share).
    pub start: Vec2,
    /// Last frame's position, so this pass computes its own deltas rather than relying on
    /// `Touch::delta()` (whose presence and semantics have moved between Bevy versions; owning it
    /// here keeps the look feel identical no matter what the engine reports).
    pub last: Vec2,
    /// Current position.
    pub pos: Vec2,
    /// Seconds since touchdown — the tap-vs-drag discriminator for [`Role::Look`].
    pub age: f32,
    /// Total path length travelled (px). A tap is *short in both* time and distance; using
    /// straight-line displacement instead would let a fast circular scrub count as a tap.
    pub travel: f32,
}

/// Every finger currently down, with its role. Rebuilt incrementally each frame by
/// [`classify_touches`] — entries are added on touchdown and removed *after* the release frame has
/// been consumed, which is what keeps a release from being dropped.
#[derive(Resource, Default)]
pub(crate) struct Fingers {
    pub active: Vec<Finger>,
}

impl Fingers {
    fn find(&self, id: u64) -> Option<&Finger> {
        self.active.iter().find(|f| f.id == id)
    }
    fn has_role(&self, role: Role) -> bool {
        self.active.iter().any(|f| f.role == role)
    }
}

/// The joystick's output for this frame, in the same shape the movement decoder already speaks.
///
/// Read by [`crate::player::input::move_axes`] as an extra *held* source, exactly the way
/// `/follow` enters as the forward term (decision 0890): synthesized input rides the HELD state
/// and never the key-down edge, so it can't trip the autorun cancel set. Toggle autorun, push the
/// stick forward, and autorun survives — same as holding W does not cancel it.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct TouchMove {
    /// Digital W.
    pub forward: bool,
    /// Digital S.
    pub backward: bool,
    /// Digital Q (strafe left) — see [`JoystickCfg::strafe_mode`] for why strafe and not turn.
    pub strafe_left: bool,
    /// Digital E (strafe right).
    pub strafe_right: bool,
    /// The raw analog vector, `-1..=1` per axis, y-up. Kept even though the movement model is
    /// digital: the gait/animation layer can taper a walk off it later without re-deriving the
    /// stick, and it is what the visuals draw the knob from.
    pub analog: Vec2,
    /// Is a finger on the stick right now? (Drives the visuals' opacity and knob return.)
    pub engaged: bool,
}

/// Camera-look delta accumulated from every [`Role::Look`] finger this frame, in logical px —
/// deliberately the same unit and sign convention as `AccumulatedMouseMotion::delta`, so the
/// camera applies it through the *existing* yaw/pitch integrator instead of growing a second one.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct TouchLook {
    pub delta: Vec2,
    /// A look finger is down — the camera treats this as "a look button is held", which is what
    /// makes free-swipe work without a mouse button to hold.
    pub active: bool,
    /// How many [`Role::Look`] fingers are down this frame. The camera reads this to pick which
    /// mouse button a swipe stands in for: 1 finger turns the body (right-click's job), 2+ fingers
    /// orbit the camera only (left-click's job) — see `player::camera`'s own doc on the split.
    pub finger_count: u8,
}

/// The UI pointer as touch sees it. The UI pass reads this **instead of** re-deriving a touch from
/// [`Touches`], and — critically — `pos` stays valid on the release frame.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct TouchPointer {
    /// Logical px, y-down. `None` when no UI finger is down *and* none released this frame.
    pub pos: Option<Vec2>,
    /// The UI finger came down this frame.
    pub just_pressed: bool,
    /// The UI finger lifted this frame. Reported on the same frame `pos` still resolves, so the
    /// press/release pair always completes.
    pub just_released: bool,
}

/// A world tap: a [`Role::Look`] finger that lifted without really moving. This is how you select
/// a mob without a mouse, and it is why look-drag and world-click can share one finger — the same
/// arbitration the desktop client does between "right-drag to turn" and "right-click to interact".
#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct TouchWorldTap {
    /// Logical px, y-down.
    pub pos: Vec2,
}

/// Tunables. Every one is an env var so a device can be dialled in without a rebuild — the same
/// posture as the rest of the `WOW_*` surface.
#[derive(Resource, Clone, Copy)]
pub(crate) struct JoystickCfg {
    /// Fraction of window width, from the left edge, inside which a touchdown claims the stick.
    pub zone_w: f32,
    /// Fraction of window height, from the bottom edge, likewise.
    pub zone_h: f32,
    /// Radius (logical px) at which the stick reads full deflection.
    pub radius: f32,
    /// Fraction of `radius` below which the stick reads zero — kills thumb tremor.
    pub dead_zone: f32,
    /// Look sensitivity multiplier applied to the raw px delta before the camera sees it.
    pub look_scale: f32,
    /// A look finger lifting under both of these is a world tap, not a drag.
    pub tap_max_secs: f32,
    pub tap_max_px: f32,
    /// `true` (the default): stick left/right **strafes**. The vanilla keyboard model turns with
    /// A/D unless mouse-looking — but on touch, look is a separate always-available gesture, so
    /// turning with the stick as well would fight the swipe and make the avatar spin under the
    /// camera. Strafing is what every touch port of this control scheme settles on. Set
    /// `WOW_TOUCH_STRAFE=0` for the literal A/D-turn behaviour.
    pub strafe_mode: bool,
    /// Master switch. Defaults on for Android, off elsewhere, and force-on with `WOW_TOUCH=1`
    /// (which is how you test the whole layer on a desktop box with a touchscreen).
    pub enabled: bool,
}

impl Default for JoystickCfg {
    fn default() -> Self {
        let env_f = |k: &str, d: f32| {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(d)
        };
        let env_b = |k: &str, d: bool| {
            std::env::var(k)
                .ok()
                .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
                .unwrap_or(d)
        };
        Self {
            // ~15% smaller than the original 0.45×0.60 footprint (issue's touch-enhancement ask):
            // 0.45×0.85 ≈ 0.38, 0.60×0.85 = 0.51. This only shrinks the RECTANGLE a touchdown has
            // to land inside to claim the stick — `radius` (how far the thumb then has to drag for
            // full deflection once anchored) is a separate, untouched knob; the ask was the
            // touchzone's screen footprint, not the stick's own feel.
            zone_w: env_f("WOW_TOUCH_ZONE_W", 0.38).clamp(0.05, 1.0),
            zone_h: env_f("WOW_TOUCH_ZONE_H", 0.51).clamp(0.05, 1.0),
            radius: env_f("WOW_TOUCH_RADIUS", 110.0).max(20.0),
            dead_zone: env_f("WOW_TOUCH_DEADZONE", 0.18).clamp(0.0, 0.9),
            look_scale: env_f("WOW_TOUCH_LOOK", 1.0).max(0.01),
            tap_max_secs: env_f("WOW_TOUCH_TAP_SECS", 0.35).max(0.05),
            tap_max_px: env_f("WOW_TOUCH_TAP_PX", 16.0).max(1.0),
            strafe_mode: env_b("WOW_TOUCH_STRAFE", true),
            enabled: env_b("WOW_TOUCH", cfg!(target_os = "android")),
        }
    }
}

/// Where the stick is currently anchored. The stick is **floating**: it centres wherever the thumb
/// lands inside the zone rather than sitting at a fixed spot, so you never have to look down to
/// find it. `None` when no joystick finger is down.
#[derive(Resource, Default)]
pub(crate) struct JoystickAnchor {
    pub centre: Option<Vec2>,
}

/// Ordering label — everything that *consumes* touch state runs after this.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TouchInputSet;

pub(crate) struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fingers>()
            .init_resource::<TouchMove>()
            .init_resource::<TouchLook>()
            .init_resource::<TouchPointer>()
            .init_resource::<JoystickAnchor>()
            .init_resource::<JoystickCfg>()
            .add_message::<TouchWorldTap>()
            // `Update` and not `PreUpdate`: `Touches` is refreshed by Bevy's own input pass in
            // `PreUpdate`, so running here needs no explicit `.after(InputSystems)` — every
            // `PreUpdate` system has already finished. That avoids naming an engine set whose
            // name has moved between releases, and costs nothing: the consumers are all in
            // `Update` too, ordered behind `TouchInputSet`.
            // Before the UI feed, which reads `TouchPointer`, and so before the world input that
            // runs after the UI pass.
            .configure_sets(Update, TouchInputSet.before(crate::ui_script::UiInput))
            .add_systems(
                Update,
                (classify_touches, drive_touch_ui_interactions)
                    .chain()
                    .in_set(TouchInputSet),
            )
            .add_plugins(joystick::JoystickVisualsPlugin);
    }
}

/// The arbiter. Runs once per frame, before anything reads touch state.
#[allow(clippy::too_many_arguments)]
type UiNodeQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static ComputedNode,
        &'static UiGlobalTransform,
        Option<&'static FocusPolicy>,
        Option<&'static crate::glue::widgets::GlueDisabled>,
        Option<&'static Visibility>,
        Option<&'static Interaction>,
    ),
    With<Node>,
>;

fn classify_touches(
    touches: Res<Touches>,
    time: Res<Time<Real>>,
    cfg: Res<JoystickCfg>,
    window: Query<&Window, With<PrimaryWindow>>,
    script: Option<NonSend<benilla_ui::script::UiScript>>,
    ui_nodes: UiNodeQuery,
    mut fingers: ResMut<Fingers>,
    mut mv: ResMut<TouchMove>,
    mut look: ResMut<TouchLook>,
    mut pointer: ResMut<TouchPointer>,
    mut anchor: ResMut<JoystickAnchor>,
    mut taps: MessageWriter<TouchWorldTap>,
) {
    // Reset the per-frame outputs. The *fingers* persist; their published effects do not.
    *mv = TouchMove::default();
    *look = TouchLook::default();
    *pointer = TouchPointer::default();

    let Ok(window) = window.single() else {
        fingers.active.clear();
        anchor.centre = None;
        return;
    };
    if !cfg.enabled {
        fingers.active.clear();
        anchor.centre = None;
        return;
    }

    let dt = time.delta_secs();
    let (win_w, win_h) = (window.width(), window.height());

    // ── 1. Claim newly-pressed fingers ──────────────────────────────────────────────────────
    //
    // Classification happens here and nowhere else. Note this reads `iter_just_pressed()`, which
    // is authoritative for "came down this frame" regardless of how many fingers are already
    // resting — the coin-flip that `iter().next()` used to run is gone entirely.
    for t in touches.iter_just_pressed() {
        if fingers.find(t.id()).is_some() {
            continue;
        }
        let pos = t.position();
        let in_zone = pos.x <= win_w * cfg.zone_w && pos.y >= win_h * (1.0 - cfg.zone_h);

        let role = if in_zone && !fingers.has_role(Role::Joystick) {
            // The stick wins its zone outright — see the module doc on full-screen addon frames.
            anchor.centre = Some(pos);
            Role::Joystick
        } else if hits_ui(
            script.as_deref(),
            &ui_nodes,
            window.scale_factor(),
            win_h,
            pos,
        ) {
            Role::Ui
        } else {
            Role::Look
        };

        fingers.active.push(Finger {
            id: t.id(),
            role,
            start: pos,
            last: pos,
            pos,
            age: 0.0,
            travel: 0.0,
        });
    }

    // ── 2. Advance every live finger ────────────────────────────────────────────────────────
    for f in &mut fingers.active {
        if let Some(t) = touches.get_pressed(f.id) {
            let p = t.position();
            f.last = f.pos;
            f.pos = p;
            f.travel += (p - f.last).length();
            f.age += dt;
        }
    }

    // ── 3. Publish this frame's outputs ─────────────────────────────────────────────────────
    for f in &fingers.active {
        match f.role {
            Role::Joystick => {
                let centre = anchor.centre.unwrap_or(f.start);
                // y is flipped into the movement model's y-up: screen-up must read as forward.
                let raw = Vec2::new(f.pos.x - centre.x, centre.y - f.pos.y) / cfg.radius;
                let len = raw.length();
                let v = if len <= cfg.dead_zone {
                    Vec2::ZERO
                } else {
                    // Rescale past the dead zone so the very first movement outside it is a
                    // *small* one. Without this the stick jumps straight to `dead_zone` worth of
                    // deflection the instant it engages, which reads as a twitch.
                    let scaled = ((len - cfg.dead_zone) / (1.0 - cfg.dead_zone)).min(1.0);
                    raw / len * scaled
                };
                mv.analog = v;
                mv.engaged = true;
                // Digital thresholds. 0.35 on each axis gives clean 8-way output: pushing
                // diagonally lights both axes, pushing straight lights one.
                const ON: f32 = 0.35;
                mv.forward = v.y > ON;
                mv.backward = v.y < -ON;
                if cfg.strafe_mode {
                    mv.strafe_left = v.x < -ON;
                    mv.strafe_right = v.x > ON;
                }
            }
            Role::Look => {
                look.delta += (f.pos - f.last) * cfg.look_scale;
                look.active = true;
                look.finger_count += 1;
            }
            Role::Ui => {
                pointer.pos = Some(f.pos);
                pointer.just_pressed |= touches.just_pressed(f.id);
            }
        }
    }

    // ── 4. Retire released fingers — *after* publishing their release ────────────────────────
    //
    // This is the fix for the dropped-release bug. A finger that lifted this frame is still in
    // `fingers.active` at this point, so its last known position is still available to hand to the
    // UI, and the release is reported alongside it. Only then is it dropped.
    let mut retire: Vec<u64> = Vec::new();
    for f in &fingers.active {
        if touches.just_released(f.id) || touches.get_pressed(f.id).is_none() {
            // `get_pressed().is_none()` is the belt to `just_released`'s braces: a touch cancelled
            // by the OS (a system gesture, an incoming call) never reports a release at all, and a
            // finger left in `active` forever would pin the joystick or the look session on.
            match f.role {
                Role::Ui => {
                    pointer.pos = Some(f.pos);
                    pointer.just_released = true;
                }
                Role::Look => {
                    if f.age <= cfg.tap_max_secs && f.travel <= cfg.tap_max_px {
                        taps.write(TouchWorldTap { pos: f.pos });
                    }
                }
                Role::Joystick => {
                    anchor.centre = None;
                }
            }
            retire.push(f.id);
        }
    }
    fingers.active.retain(|f| !retire.contains(&f.id));
}

/// Does a touchdown at `pos` (logical px, y-down) land on a mouse-enabled UI frame?
///
/// Mirrors the coordinate conversion the UI pass already does: flip through the window height into
/// the engine's y-up space, then divide by the seam scale into its 768-virtual units.
fn hits_ui(
    script: Option<&benilla_ui::script::UiScript>,
    ui_nodes: &UiNodeQuery,
    scale: f32,
    win_h: f32,
    pos: Vec2,
) -> bool {
    // 1. FrameXML Lua UI engine hit test
    if let Some(script) = script {
        let s = crate::ui_script::seam_scale(win_h);
        let (x, y) = (pos.x / s, (win_h - pos.y) / s);
        if script
            .hit_test(x, y)
            .is_some_and(|id| !script.is_world_frame(id))
        {
            return true;
        }
    }

    // 2. Native Bevy UI hit test (glue screens: Login, CharSelect, CharCreate, RealmSelect, Dialogs)
    for (node, transform, policy, disabled, vis, interaction) in ui_nodes {
        if disabled.is_some_and(|d| d.0) || vis.is_some_and(|v| *v == Visibility::Hidden) {
            continue;
        }
        if interaction.is_some() || policy.is_some_and(|p| *p == FocusPolicy::Block) {
            if crate::glue::node_hit_test(node, transform, scale, pos) {
                return true;
            }
        }
    }

    false
}

/// Drives `Interaction` components on Bevy UI entities from `TouchPointer`.
///
/// On touch devices without a physical mouse pointer, Bevy's built-in `ui_focus_system`
/// sees `cursor_position() == None`. This system updates `Interaction` for active Bevy UI nodes
/// touched by the user so that edit boxes focus, buttons enter pushed state, and `glue_clicks`
/// registers the release click.
#[allow(clippy::type_complexity)]
fn drive_touch_ui_interactions(
    pointer: Res<TouchPointer>,
    window: Query<&'static Window, With<PrimaryWindow>>,
    mut ui_nodes: Query<(
        Entity,
        &'static mut Interaction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        Option<&'static crate::glue::widgets::GlueDisabled>,
        Option<&'static Visibility>,
    )>,
) {
    let Some(pos) = pointer.pos else {
        return;
    };
    let Ok(window) = window.single() else {
        return;
    };
    let scale = window.scale_factor();

    for (_entity, mut interaction, node, transform, disabled, vis) in &mut ui_nodes {
        if disabled.is_some_and(|d| d.0) || vis.is_some_and(|v| *v == Visibility::Hidden) {
            if *interaction != Interaction::None {
                *interaction = Interaction::None;
            }
            continue;
        }
        if crate::glue::node_hit_test(node, transform, scale, pos) {
            let want = if pointer.just_released {
                Interaction::Hovered
            } else {
                Interaction::Pressed
            };
            if *interaction != want {
                *interaction = want;
            }
        } else if *interaction != Interaction::None {
            *interaction = Interaction::None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> JoystickCfg {
        JoystickCfg {
            zone_w: 0.45,
            zone_h: 0.60,
            radius: 100.0,
            dead_zone: 0.2,
            look_scale: 1.0,
            tap_max_secs: 0.35,
            tap_max_px: 16.0,
            strafe_mode: true,
            enabled: true,
        }
    }

    /// The dead zone rescale: just past the edge must be a *small* deflection, not a jump to 0.2.
    #[test]
    fn dead_zone_rescales_instead_of_stepping() {
        let c = cfg();
        let len = c.dead_zone + 0.01;
        let scaled = (len - c.dead_zone) / (1.0 - c.dead_zone);
        assert!(
            scaled < 0.02,
            "first step outside the dead zone must be small"
        );
    }

    /// 8-way digital output: a clean diagonal lights both axes.
    #[test]
    fn diagonal_lights_both_axes() {
        let v = Vec2::new(0.7, 0.7);
        const ON: f32 = 0.35;
        assert!(v.y > ON && v.x > ON);
    }

    /// A straight push lights one axis only.
    #[test]
    fn cardinal_push_lights_one_axis() {
        let v = Vec2::new(0.05, 0.9);
        const ON: f32 = 0.35;
        assert!(v.y > ON);
        assert!(v.x <= ON && v.x >= -ON);
    }

    /// The zone test is bottom-left: a touch top-right is never the stick.
    #[test]
    fn zone_is_bottom_left() {
        let c = cfg();
        let (w, h) = (1000.0, 800.0);
        let bottom_left = Vec2::new(100.0, 700.0);
        let top_right = Vec2::new(900.0, 100.0);
        let inz = |p: Vec2| p.x <= w * c.zone_w && p.y >= h * (1.0 - c.zone_h);
        assert!(inz(bottom_left));
        assert!(!inz(top_right));
    }

    /// A slow scrub that ends where it started is a drag, not a tap — distance is measured along
    /// the path, not end-to-end.
    #[test]
    fn travel_is_path_length_not_displacement() {
        let c = cfg();
        let mut travel = 0.0;
        let pts = [
            Vec2::new(0.0, 0.0),
            Vec2::new(40.0, 0.0),
            Vec2::new(40.0, 40.0),
            Vec2::new(0.0, 0.0),
        ];
        for w in pts.windows(2) {
            travel += (w[1] - w[0]).length();
        }
        assert!(
            travel > c.tap_max_px,
            "a round trip must not read as a stationary tap"
        );
    }

    #[test]
    fn touch_on_bevy_ui_node_classifies_as_role_ui_and_sets_pressed() {
        use crate::glue::GlueClicks;
        use bevy::input::touch::{TouchInput, TouchPhase};

        let mut app = App::new();
        app.add_plugins((
            bevy::input::InputPlugin,
            bevy::window::WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            },
        ))
        .init_resource::<GlueClicks>()
        .add_plugins(TouchPlugin)
        .add_systems(Update, crate::glue::glue_clicks);

        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: bevy::window::WindowResolution::new(1000, 800),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();

        // Enable touch mode in test
        app.world_mut().resource_mut::<JoystickCfg>().enabled = true;

        // Spawn a Bevy UI button at (500, 400)
        let button = app
            .world_mut()
            .spawn((
                Node::default(),
                Button,
                Interaction::None,
                ComputedNode::default(),
                UiGlobalTransform::default(),
            ))
            .id();

        // Simulate touchdown at (500, 400)
        let touch_location = Vec2::new(500.0, 400.0);
        app.world_mut().write_message(TouchInput {
            phase: TouchPhase::Started,
            position: touch_location,
            force: None,
            id: 1,
            window,
        });

        app.update();

        // 1. Verify touch classification
        let fingers = app.world().resource::<Fingers>();
        assert_eq!(fingers.active.len(), 1);
        assert_eq!(fingers.active[0].role, Role::Ui);

        let pointer = app.world().resource::<TouchPointer>();
        assert_eq!(pointer.pos, Some(touch_location));
        assert!(pointer.just_pressed);

        // 2. Verify Interaction changed to Pressed
        let interaction = *app.world().get::<Interaction>(button).unwrap();
        assert_eq!(interaction, Interaction::Pressed);

        // Simulate touch release
        app.world_mut().write_message(TouchInput {
            phase: TouchPhase::Ended,
            position: touch_location,
            force: None,
            id: 1,
            window,
        });

        app.update();

        // 3. Verify release registered a click in GlueClicks
        let clicks = app.world().resource::<GlueClicks>();
        assert!(
            clicks.hit(button),
            "touch release must register in GlueClicks"
        );
    }
}
