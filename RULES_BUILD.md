# RULES_BUILD.md — Aturan Wajib Build Nexora Finder

> **PANDUAN UNTUK AI CODING ASSISTANT:** Baca dokumen ini SEBELUM melakukan build apa pun
> (binary standalone, installer, atau rilis update).

---

## ⚠️ Aturan #1 — JANGAN pakai `cargo build --release` polos

Tauri v2 hanya menanamkan aset frontend (`dist/`) ke dalam `.exe` bila fitur
`tauri/custom-protocol` aktif. Tanpa fitur itu, binary akan memuat **`http://localhost:1420`**
(dev server). Akibatnya jendela tampil **kosong abu-abu dengan scrollbar `◄ ► ▲ ▼`**.

| Perintah | Hasil |
|---|---|
| `cargo build --release` | ❌ RUSAK — memuat localhost:1420 |
| `cargo build --release --features tauri/custom-protocol` | ✅ Benar |
| `npm.cmd run tauri build` | ✅ Benar (CLI otomatis menambah fitur) |
| `npm.cmd run tauri dev` | ✅ Untuk development (hot reload) |

---

## Aturan #2 — Build standalone wajib lewat script

```powershell
powershell -ExecutionPolicy Bypass -File .\build-standalone.ps1
```
Script menjalankan: `npm run build` → cargo + `custom-protocol` → stop proses lama → salin ke
`d:\Rust\Nexora Finder\nexora-finder.exe`.

## Aturan #3 — Build installer / rilis update wajib lewat Tauri CLI

```powershell
npm.cmd run tauri build -- --bundles nsis
```
- Updater (`latest.json` + `.sig`) hanya valid untuk installer hasil Tauri CLI.
- Ikuti `RULES_VERSION_RELEASE.md` untuk bump versi, signing, dan upload release GitHub.
- Endpoint updater: `https://github.com/nexora-ctrl/nexora-finder/releases/latest/download/latest.json`
  — `latest.json` harus di-upload sebagai asset release terbaru.

## Aturan #4 — Jangan ubah resize jendela dengan Win32 `SetWindowPos` manual

Gunakan `window.set_size()` / `set_position()` Tauri agar viewport WebView2 tetap sinkron.

## Aturan #5 — Verifikasi setelah build

1. Ukuran `nexora-finder.exe` harus ±14,9 MB (aset tertanam). ±14,5 MB = kemungkinan tanpa aset.
2. Jalankan exe → kolom pencarian harus tampil (bukan kotak abu-abu kosong).
3. Uji tombol **Periksa Pembaruan** di Pengaturan.

## Aturan #6 — npm di PowerShell

Selalu gunakan `npm.cmd` (bukan `npm`) karena execution policy memblokir `npm.ps1`.
