import { Bookmark, Factory, Hand, Keyboard, RefreshCw, RotateCcw, TriangleAlert } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { Button, ConfirmDialog, Dialog, Panel, Spinner } from "@/components/ui";
import { toIpcError } from "@/lib/errors";
import { loadKeyboardLayout, MEDIA_KEYS, modifiersOf, shortcutLabel, usageForCode } from "@/lib/hidKeys";
import { api } from "@/lib/ipc";
import { cn } from "@/lib/cn";
import { selectActiveProfile, useProfiles } from "@/stores/profiles";
import { toast } from "@/stores/toasts";
import type { ButtonAction, ButtonBinding, ButtonDefaults, ButtonLayout, MouseButton } from "@/types/buttons";
import type { DeviceInfo } from "@/types/device";

import { actionLabel, MOUSE_BUTTON_LABEL, sameAction, SPECIAL_LABEL } from "./buttonActions";
import { diagramFor } from "./mouseDiagrams";
import { MouseDiagramView } from "./MouseDiagramView";

type State =
  | { status: "loading" }
  | { status: "unsupported" }
  | { status: "error"; message: string }
  | { status: "ready"; layout: ButtonLayout };

type Restore = "factory" | "mine";

/**
 * What each button does, read from the device. Assignments are stored in
 * the device's own memory, so they work without PeriphOS running.
 */
export function ButtonsPanel({ device }: { device: DeviceInfo }) {
  const [state, setState] = useState<State>({ status: "loading" });
  const [defaults, setDefaults] = useState<ButtonDefaults | null>(null);
  const [editing, setEditing] = useState<ButtonBinding | null>(null);
  const [restoring, setRestoring] = useState<Restore | null>(null);
  const [saving, setSaving] = useState(false);
  const [releasing, setReleasing] = useState(false);
  const activeProfile = useProfiles(selectActiveProfile);
  /** Whether the active profile has its own assignments for this device. */
  const [ownButtons, setOwnButtons] = useState(false);

  const load = useCallback(async (quiet = false) => {
    if (!quiet) setState({ status: "loading" });
    try {
      setState({ status: "ready", layout: await api.deviceButtons(device.id) });
    } catch (e) {
      const error = toIpcError(e);
      setState(error.code === "unsupported" ? { status: "unsupported" } : { status: "error", message: error.message });
      return;
    }
    try {
      setDefaults(await api.buttonDefaults(device.id));
    } catch {
      setDefaults(null);
    }
    const profile = selectActiveProfile(useProfiles.getState());
    if (profile) {
      try {
        const saved = await api.profileDeviceSettings(profile.id);
        setOwnButtons(saved.some((e) => e.deviceId === device.id && e.buttons !== null));
      } catch {
        setOwnButtons(false);
      }
    }
  }, [device.id]);

  useEffect(() => {
    void load();
  }, [load]);

  // Another profile was applied: the mouse may now have other assignments.
  const appliedRevision = useProfiles((s) => s.appliedRevision);
  const firstRevision = useRef(appliedRevision);
  useEffect(() => {
    if (appliedRevision !== firstRevision.current) void load(true);
  }, [appliedRevision, load]);

  /** Stores the buttons in the active profile; `null` makes it follow the defaults. */
  const saveToProfile = async (buttons: ButtonAction[] | null) => {
    const profile = selectActiveProfile(useProfiles.getState());
    if (!profile) return;
    try {
      await api.saveProfileButtons(profile.id, device.id, device.name, buttons);
      setOwnButtons(buttons !== null);
    } catch (e) {
      toast.error(`The buttons were changed on the mouse but not saved to "${profile.name}": ${toIpcError(e).message}`);
    }
  };

  const layout = state.status === "ready" ? state.layout : null;
  const diagram = layout ? diagramFor(device.name, layout.buttons.length) : null;

  const modified = useMemo(() => {
    const changed = new Set<number>();
    layout?.factoryDefaults?.forEach((action, i) => {
      const current = layout.buttons[i];
      if (current && !sameAction(current.action, action)) changed.add(i);
    });
    return changed;
  }, [layout]);

  /** Writes the assignments; returns whether all of them were kept. */
  const write = async (
    assignments: { index: number; action: ButtonAction }[],
    followDefaults = false,
  ): Promise<boolean> => {
    const next = await api.setDeviceButtons(device.id, assignments);
    setState({ status: "ready", layout: next });
    await saveToProfile(followDefaults ? null : next.buttons.map((b) => b.action));
    return assignments.every(
      ({ index, action }) => action.type === "other" || (next.buttons[index] && sameAction(next.buttons[index].action, action)),
    );
  };

  const assign = async (binding: ButtonBinding, action: ButtonAction) => {
    setSaving(true);
    try {
      const kept = await write([{ index: binding.index, action }]);
      setEditing(null);
      if (kept) toast.success(`${binding.name} now does: ${actionLabel(action)}.`);
      else toast.error(`${device.name} did not keep the new assignment for ${binding.name}.`);
    } catch (e) {
      toast.error(toIpcError(e).message);
    } finally {
      setSaving(false);
    }
  };

  const restore = async (which: Restore) => {
    const actions = which === "factory" ? layout?.factoryDefaults : defaults?.actions;
    if (!actions) return;
    const kept = await write(
      actions.map((action, index) => ({ index, action })),
      which === "mine",
    );
    if (!kept) throw new Error(`${device.name} did not keep every assignment. Read the buttons again to see what it has now.`);
    toast.success(which === "factory" ? "Factory button settings restored." : "Your default buttons restored.");
  };

  const saveAsDefaults = async () => {
    if (!layout) return;
    setSaving(true);
    try {
      setDefaults(await api.saveButtonDefaults(device.id, device.name, layout.buttons.map((b) => b.action)));
      // The active profile now matches the defaults: let it follow them.
      await saveToProfile(null);
      toast.success(`Saved as the default buttons for ${device.name}. Profiles without their own buttons use them.`);
    } catch (e) {
      toast.error(toIpcError(e).message);
    } finally {
      setSaving(false);
    }
  };

  /** Forgets the defaults and the active profile's buttons, so PeriphOS leaves them alone. */
  const release = async () => {
    await api.deleteButtonDefaults(device.id);
    setDefaults(null);
    await saveToProfile(null);
    toast.success(`PeriphOS no longer changes the buttons of ${device.name} in this profile.`);
  };

  if (state.status === "unsupported") return null;

  return (
    <Panel
      title="Buttons"
      description={
        <>
          Changes are saved to the active profile,{" "}
          <span className="font-semibold text-fg">{activeProfile?.name ?? "…"}</span>, and stored in the mouse
          {layout?.onboardProfile != null ? ` (onboard profile ${layout.onboardProfile})` : ""}, so they keep
          working without PeriphOS.{" "}
          {ownButtons
            ? "This profile has its own buttons."
            : defaults
              ? "This profile uses your default buttons."
              : "Profiles without their own buttons use your defaults, once you save them."}
        </>
      }
      actions={
        <Button
          size="sm"
          variant="ghost"
          icon={<RefreshCw className="size-3.5" />}
          onClick={() => void load()}
          disabled={state.status === "loading" || saving}
        >
          Read again
        </Button>
      }
    >
      {state.status === "loading" && (
        <p className="flex items-center gap-2 text-[13px] text-muted">
          <Spinner className="size-3.5" /> Reading buttons from the device…
        </p>
      )}
      {state.status === "error" && (
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-[13px] text-danger">{state.message}</p>
          <Button size="sm" onClick={() => void load()}>
            Try again
          </Button>
        </div>
      )}
      {layout && (
        <div className="flex flex-col gap-5">
          {!layout.inEffect && (
            <p className="flex gap-2.5 rounded-2xl bg-warning/10 px-4 py-3 text-[13px] text-warning">
              <TriangleAlert className="mt-0.5 size-4 shrink-0" />
              <span>
                The mouse is not using its onboard memory right now (the manufacturer&apos;s app
                switched it off), so these assignments only take effect once that mode is on again.
              </span>
            </p>
          )}

          {diagram && (
            <div className="hidden lg:block">
              <MouseDiagramView
                diagram={diagram}
                buttons={layout.buttons}
                modified={modified}
                disabled={saving}
                onSelect={setEditing}
              />
            </div>
          )}
          <ul className={cn("flex flex-col", diagram && "lg:hidden")}>
            {layout.buttons.map((binding) => (
              <li
                key={binding.index}
                className="flex items-center justify-between gap-4 border-t border-line py-2.5 first:border-t-0 first:pt-0"
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium">{binding.name}</p>
                  <p className={cn("text-[13px]", binding.action.type === "disabled" ? "text-subtle" : "text-muted")}>
                    {actionLabel(binding.action)}
                    {modified.has(binding.index) && <span className="ml-2 text-accent">Changed</span>}
                  </p>
                </div>
                <Button size="sm" onClick={() => setEditing(binding)} disabled={saving}>
                  Change
                </Button>
              </li>
            ))}
          </ul>

          <div className="flex flex-wrap items-center gap-2 border-t border-line pt-4">
            {layout.factoryDefaults && (
              <Button
                size="sm"
                icon={<Factory className="size-3.5" />}
                disabled={saving || modified.size === 0}
                onClick={() => setRestoring("factory")}
              >
                Restore factory settings
              </Button>
            )}
            <Button
              size="sm"
              icon={<RotateCcw className="size-3.5" />}
              disabled={saving || !defaults}
              onClick={() => setRestoring("mine")}
              title={defaults ? undefined : "Save your defaults first."}
            >
              Restore my defaults
            </Button>
            <Button size="sm" variant="ghost" icon={<Bookmark className="size-3.5" />} disabled={saving} onClick={() => void saveAsDefaults()}>
              Save current as my defaults
            </Button>
            {(defaults || ownButtons) && (
              <Button size="sm" variant="ghost" icon={<Hand className="size-3.5" />} disabled={saving} onClick={() => setReleasing(true)}>
                Stop managing buttons
              </Button>
            )}
            <p className="basis-full text-[12px] text-subtle">
              {defaults
                ? `Your defaults were saved on ${new Date(defaults.updatedAt).toLocaleDateString()}. New profiles, and profiles without their own buttons, use them.`
                : "Your defaults are used by new profiles and by every profile without its own buttons."}
            </p>
          </div>
        </div>
      )}
      {editing && layout && (
        <AssignDialog
          binding={editing}
          layout={layout}
          myDefault={defaults?.actions[editing.index] ?? null}
          saving={saving}
          onClose={() => setEditing(null)}
          onAssign={(action) => void assign(editing, action)}
        />
      )}
      <ConfirmDialog
        open={releasing}
        onClose={() => setReleasing(false)}
        title="Stop managing buttons?"
        description={`PeriphOS forgets your default buttons for ${device.name} and stops setting them in "${activeProfile?.name ?? "this profile"}". The mouse keeps what it has now, also changes made with other software. Profiles that have their own buttons still set them.`}
        confirmLabel="Stop managing"
        onConfirm={release}
      />
      <ConfirmDialog
        open={restoring !== null}
        onClose={() => setRestoring(null)}
        title={restoring === "factory" ? "Restore factory button settings?" : "Restore your default buttons?"}
        description={
          restoring === "factory"
            ? `Every button of ${device.name} goes back to what it did out of the box, in onboard profile ${layout?.onboardProfile ?? ""}.`
            : `Every button of ${device.name} goes back to the assignments you saved as your defaults.`
        }
        confirmLabel="Restore"
        onConfirm={() => restore(restoring ?? "factory")}
      />
    </Panel>
  );
}

function Choice({
  selected,
  onClick,
  children,
}: {
  selected: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className={cn(
        "rounded-xl border px-3 py-1.5 text-[13px] font-medium transition-colors",
        selected ? "border-accent bg-accent/15 text-fg" : "border-line bg-raised text-muted hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}

function Group({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-2">
      <p className="text-[13px] font-semibold text-muted">{title}</p>
      <div className="flex flex-wrap gap-1.5">{children}</div>
    </div>
  );
}

const MOUSE_BUTTONS: MouseButton[] = ["left", "right", "middle", "back", "forward"];

function AssignDialog({
  binding,
  layout,
  myDefault,
  saving,
  onClose,
  onAssign,
}: {
  binding: ButtonBinding;
  layout: ButtonLayout;
  myDefault: ButtonAction | null;
  saving: boolean;
  onClose: () => void;
  onAssign: (action: ButtonAction) => void;
}) {
  const [choice, setChoice] = useState<ButtonAction>(binding.action);
  const [capturing, setCapturing] = useState(false);
  // The keyboard layout may have changed since it was read: key names follow it.
  const [, setLayoutRead] = useState(0);
  useEffect(() => {
    void loadKeyboardLayout().then(() => setLayoutRead((n) => n + 1));
  }, []);
  const pick = (action: ButtonAction) => {
    setCapturing(false);
    setChoice(action);
  };
  const is = (action: ButtonAction) => sameAction(choice, action);
  const factoryAction = layout.factoryDefaults?.[binding.index];
  const factory = factoryAction && factoryAction.type !== "other" ? factoryAction : null;

  return (
    <Dialog
      open
      onClose={onClose}
      title={`Assign ${binding.name}`}
      description={`Now: ${actionLabel(binding.action)}`}
      className="w-[min(560px,calc(100vw-32px))]"
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="primary"
            loading={saving}
            disabled={choice.type === "other" || sameAction(choice, binding.action)}
            onClick={() => onAssign(choice)}
          >
            Save to mouse
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-5">
        {(factory || myDefault) && (
          <Group title="Defaults">
            {factory && (
              <Choice selected={is(factory)} onClick={() => pick(factory)}>
                Factory: {actionLabel(factory)}
              </Choice>
            )}
            {myDefault && myDefault.type !== "other" && (
              <Choice selected={is(myDefault)} onClick={() => pick(myDefault)}>
                Mine: {actionLabel(myDefault)}
              </Choice>
            )}
          </Group>
        )}
        <Group title="Mouse">
          {MOUSE_BUTTONS.map((button) => (
            <Choice key={button} selected={is({ type: "mouse", button })} onClick={() => pick({ type: "mouse", button })}>
              {MOUSE_BUTTON_LABEL[button]}
            </Choice>
          ))}
        </Group>

        <Group title="Keyboard">
          <button
            type="button"
            onClick={() => setCapturing(true)}
            onBlur={() => setCapturing(false)}
            onKeyDown={(e) => {
              if (!capturing) return;
              e.preventDefault();
              e.stopPropagation();
              const usage = usageForCode(e.code);
              if (usage === null) return;
              setChoice({ type: "key", key: usage, modifiers: modifiersOf(e, usage) });
              setCapturing(false);
            }}
            className={cn(
              "flex items-center gap-2 rounded-xl border px-3 py-1.5 text-[13px] font-medium",
              capturing || choice.type === "key"
                ? "border-accent bg-accent/15 text-fg"
                : "border-line bg-raised text-muted hover:text-fg",
            )}
          >
            <Keyboard className="size-3.5" />
            {capturing
              ? "Press a key or shortcut…"
              : choice.type === "key"
                ? shortcutLabel(choice.key, choice.modifiers)
                : "Choose a key"}
          </button>
        </Group>

        <Group title="Media">
          {MEDIA_KEYS.map((m) => (
            <Choice key={m.usage} selected={is({ type: "media", usage: m.usage })} onClick={() => pick({ type: "media", usage: m.usage })}>
              {m.label}
            </Choice>
          ))}
        </Group>

        <Group title="Mouse functions">
          {layout.specialFunctions.map((f) => (
            <Choice key={f} selected={is({ type: "special", function: f })} onClick={() => pick({ type: "special", function: f })}>
              {SPECIAL_LABEL[f]}
            </Choice>
          ))}
        </Group>

        <Group title="Other">
          <Choice selected={is({ type: "disabled" })} onClick={() => pick({ type: "disabled" })}>
            Do nothing
          </Choice>
        </Group>

        {binding.action.type === "other" && (
          <p className="text-[13px] text-muted">
            This button has an assignment PeriphOS cannot edit yet ({binding.action.description}). Choosing
            something else replaces it.
          </p>
        )}
      </div>
    </Dialog>
  );
}
