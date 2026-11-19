/**
 * Example devices for UI development in the browser preview, enabled with
 * `npm run dev` and `?fixtures` in the URL. Never included in production
 * builds and never shown in the desktop app. Keep in sync with
 * crates/periphos-hid/devices.json when changing the device model.
 */
import type { DeviceInfo, DeviceSettings } from "@/types/device";

export const PREVIEW_FIXTURE_DEVICES: DeviceInfo[] = [
  {
    id: "hid-generic:046d:c098:0",
    driverId: "hid-generic",
    name: "G502 X LIGHTSPEED",
    manufacturer: "Logitech",
    vendorId: 0x046d,
    productId: 0xc098,
    kind: "mouse",
    connection: "usb",
    knownModel: true,
    interfaceCount: 3,
    battery: null,
    connectedVia: null,
    note: "Connected with its USB cable. Onboard settings use Logitech's HID++ protocol, which PeriphOS does not implement yet.",
    capabilities: [
      { kind: { type: "buttonRemap", programmable: 13 }, support: "vendorProtocol" },
      { kind: { type: "dpi", min: 100, max: 25600, step: null }, support: "vendorProtocol" },
      { kind: { type: "pollingRate", maxHz: 1000 }, support: "vendorProtocol" },
      { kind: { type: "onboardProfiles", slots: 5 }, support: "vendorProtocol" },
      { kind: { type: "battery" }, support: "vendorProtocol" },
    ],
  },
  {
    id: "hid-generic:31e3:1402:0",
    driverId: "hid-generic",
    name: "80HE",
    manufacturer: "Wooting",
    vendorId: 0x31e3,
    productId: 0x1402,
    kind: "keyboard",
    connection: "usb",
    knownModel: true,
    interfaceCount: 6,
    battery: null,
    connectedVia: null,
    note: "Settings are stored on the keyboard and configured through Wooting's protocol, which PeriphOS does not implement yet.",
    capabilities: [
      {
        kind: { type: "analogKeys" },
        support: "vendorProtocol",
        note: "Actuation point (0.1 to 4.0 mm) and rapid trigger.",
      },
      { kind: { type: "keyRemap" }, support: "vendorProtocol" },
      { kind: { type: "pollingRate", maxHz: 8000 }, support: "vendorProtocol" },
      { kind: { type: "lighting", perKey: true, zones: null }, support: "vendorProtocol" },
    ],
  },
  {
    id: "hid-generic:046d:c232:0",
    driverId: "hid-generic",
    name: "G HUB virtual keyboard",
    manufacturer: "Logitech",
    vendorId: 0x046d,
    productId: 0xc232,
    kind: "keyboard",
    connection: "virtual",
    knownModel: true,
    interfaceCount: 1,
    battery: null,
    connectedVia: null,
    note: "Created by Logitech G HUB to send keystrokes from macros and key assignments. It is not a physical device.",
    capabilities: [],
  },
  {
    id: "hid-generic:1234:5678:0",
    driverId: "hid-generic",
    name: "USB Gaming Controller",
    manufacturer: "Generic",
    vendorId: 0x1234,
    productId: 0x5678,
    kind: "controller",
    connection: "bluetooth",
    knownModel: false,
    interfaceCount: 1,
    battery: null,
    connectedVia: null,
    note: null,
    capabilities: [],
  },
  {
    id: "logitech-hidpp:c547:0:1",
    driverId: "logitech-hidpp",
    name: "G502 X LIGHTSPEED",
    manufacturer: "Logitech",
    vendorId: 0x046d,
    productId: 0,
    kind: "mouse",
    connection: "receiver",
    knownModel: false,
    interfaceCount: 0,
    battery: { percent: 64, level: null, charging: "discharging", estimated: true },
    connectedVia: "LIGHTSPEED receiver",
    note: "Identified through Logitech's HID++ protocol. Sensitivity and report rate can be changed below; other settings cannot be changed from PeriphOS yet.",
    capabilities: [
      { kind: { type: "dpi", min: 100, max: 25600, step: null }, support: "native" },
      { kind: { type: "pollingRate", maxHz: 1000 }, support: "native" },
      { kind: { type: "battery" }, support: "native" },
    ],
  },
];

/** Simulated live settings for the receiver-connected fixture mouse. */
export function previewFixtureSettings(): Map<string, DeviceSettings> {
  const dpiValues = Array.from({ length: 511 }, (_, i) => 100 + i * 50);
  return new Map([
    [
      "logitech-hidpp:c547:0:1",
      {
        dpi: { current: 1600, values: dpiValues },
        reportRate: { currentHz: 1000, valuesHz: [125, 250, 500, 1000] },
        onboardProfileActive: false,
      },
    ],
  ]);
}
