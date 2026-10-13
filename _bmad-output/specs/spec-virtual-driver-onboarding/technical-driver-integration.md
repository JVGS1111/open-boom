# Technical Driver Integration: Windows Audio Endpoints, 7.1 Mask & Onboarding

## 1. Windows Core Audio Endpoint Customization

### 1.1 Property Keys (Core Audio API)
Windows stores endpoint metadata in the MMDevice property store (`IPropertyStore`):
* **Friendly Name:**
  * Property Key: `PKEY_Device_FriendlyName`
  * GUID: `{a45c254e-df1c-4efd-8020-67d146a850e0}, 14`
  * Value format: `VT_LPWSTR`
  * Target String: `"OpenBoom Spatial Audio (powered by VB-Cable)"`
* **Speaker Configuration Mask:**
  * Property Key: `PKEY_AudioEndpoint_PhysicalSpeakers`
  * GUID: `{1da5d803-d346-45ab-9f40-8c2f1083a47e}, 3`
  * Value format: `VT_UI4`
  * Value for 7.1 Surround: `0x0000063F` (Hex) / `1599` (Decimal)
  * Bitmask breakdown (`KSAUDIO_SPEAKER_7POINT1_SURROUND`):
    * `SPEAKER_FRONT_LEFT` (0x1)
    * `SPEAKER_FRONT_RIGHT` (0x2)
    * `SPEAKER_FRONT_CENTER` (0x4)
    * `SPEAKER_LOW_FREQUENCY` (0x8)
    * `SPEAKER_BACK_LEFT` (0x10)
    * `SPEAKER_BACK_RIGHT` (0x20)
    * `SPEAKER_SIDE_LEFT` (0x200)
    * `SPEAKER_SIDE_RIGHT` (0x400)
    * Sum: `0x1 | 0x2 | 0x4 | 0x8 | 0x10 | 0x20 | 0x200 | 0x400 = 0x63F`

### 1.2 Access & Privilege Flow
* Reading properties can be done in user mode via `IMMDevice::OpenPropertyStore(STGM_READ)`.
* Writing properties can be done via elevated IPC or registry keys under:
  `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\{Device-GUID}\Properties`
* When modifying the registry or property store, restart or notify the Windows Audio service (`AudioSrv`) if necessary to reload endpoint descriptors.

---

## 2. First-Run Gating & Onboarding Modal Flow

### 2.1 Driver Presence Check on Boot
1. On frontend load, query `get_audio_devices` or specialized IPC `check_virtual_driver_status`.
2. If `has_virtual_driver === false`:
   * Set `isDriverMissing = true` in app state.
   * Lock all audio interaction buttons (Master Power Switch, Volume Slider, Soundstage Dial, EQ Studio, Effects Rack) by applying `opacity-40 pointer-events-none select-none` with visual tooltip: *"Requer instalação do driver de áudio surround"*.
   * Automatically open the `<DriverOnboardingModal />`.

### 2.2 Onboarding Modal Specifications
* **Visual Style:** Fluent Dark modal with cyan accent, warning icon, and clear explanation.
* **Primary Action:** Button `"Instalar Dispositivo de Áudio Surround (1 Clique)"` with spinner/progress state.
* **Secondary Action:** Text link `"Fazer download manual do driver"` pointing to the official URL.

---

## 3. Driver Installation, Error Handling (Try/Catch) & Fallback

### 3.1 1-Click Silent Installation Process
* Executable: `VBCABLE_Setup_x64.exe`
* Invocation via elevated PowerShell or Rust `Command`:
  ```powershell
  Start-Process -FilePath "VBCABLE_Setup_x64.exe" -ArgumentList "-i", "-h" -Verb RunAs -Wait
  ```
  * `-i`: Install driver
  * `-h`: Silent mode (hide UI wizard)
  * `RunAs`: Triggers standard Windows UAC elevation prompt once.

### 3.2 Try/Catch Error Handling & Recovery States
Backend Tauri command `install_virtual_driver` encapsulates execution in a `Result<DriverInstallResult, String>`:
1. **Network Error (Download Failed):**
   * UI catches error: *"Não foi possível baixar o instalador do driver. Verifique sua conexão com a internet."*
   * Displays `"Tentar Novamente"` and manual download button.
2. **UAC Elevation Cancelled (Exit Code != 0 / User Denied):**
   * UI catches error: *"A permissão de administrador (UAC) foi cancelada. O driver não pôde ser instalado."*
   * Displays `"Tentar Novamente"` and `"Baixar Manualmente"`.
3. **Manual Download Link:**
   * URL: `https://vb-audio.com/Cable/`
   * Open via system default browser using Tauri shell API `open("https://vb-audio.com/Cable/")`.

---

## 4. UI Branding & Purge Matrix

| Old UI Label | New UI Label | Motivation |
| :--- | :--- | :--- |
| `Driver Boom Audio (Modo A)` | `Driver de Áudio Surround 7.1 (Modo A)` | Zero brand confusion, professional neutral naming |
| `Boom Audio` (Dropdowns) | `OpenBoom Spatial Audio (powered by VB-Cable)` | Explicit attribution without trademark infringement |
| `Boom Audio Bridge` | `Virtual Audio Sink Bridge` | Technical clarity |

---

## 5. Live Host Smoke Test Protocol

1. Open Windows Device Manager (`devmgmt.msc`) -> Sound, video and game controllers.
2. Uninstall existing VB-Audio Virtual Cable (or `boomvad.sys`) if present.
3. Launch OpenBoom:
   * Verify all controls are grayed out.
   * Verify Driver Onboarding Modal opens automatically.
4. Test Failure Path:
   * Click install and click "Não" (Cancel) on the Windows UAC prompt.
   * Verify UI shows clear error alert and manual download link.
5. Test Success Path:
   * Click install and click "Sim" (Approve) on UAC.
   * Verify driver installs silently in < 15s.
   * Verify endpoint appears as `OpenBoom Spatial Audio (powered by VB-Cable)`.
   * Verify channel mask is 7.1 (`0x63F`).
   * Verify controls are immediately enabled and spatial sound works without echo.
