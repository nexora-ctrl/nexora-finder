# RULES_VERSION_RELEASE.md — SOP Update Versi & Rilis Installer

> **PANDUAN UNTUK AI CODING ASSISTANT:**
> Gunakan dokumen ini sebagai Standar Operasional Prosedur (SOP) otomatis ketika Pengguna (User) meminta untuk:
> - Mengupdate / menaikkan versi aplikasi (misal: dari `0.1.0` ke `0.2.0-beta`).
> - Membuat paket installer terbaru (`.msi` / `.exe`).
> - Mempersiapkan rilis update lengkap beserta sinkronisasi Git.

---

## 📋 DAFTAR LANGKAH WAJIB (CHECKLIST RILIS VERSI BARU)

Ketika user meminta rilis atau update versi baru (contoh: *"update ke versi 0.2.0-beta dan buatkan installernya"*), AI wajib menjalankan langkah-langkah berikut secara berurutan:

### 1. Sinkronisasi Nomor Versi di Seluruh File Proyek
Pastikan versi lama diganti ke versi baru pada ketiga file utama ini:
- **`src-tauri/Cargo.toml`**:
  ```toml
  [package]
  version = "0.2.0" # sesuaikan semver
  ```
- **`src-tauri/tauri.conf.json`**:
  ```json
  "version": "0.2.0"
  ```
- **`src/components/SettingsModal.tsx`**:
  ```tsx
  <span className="version-tag">Nexora Finder • Beta v0.2.0</span>
  ```

---

### 2. Kompilasi & Build Paket Installer Baru
Jalankan proses pembuatan installer NSIS/MSI melalui terminal:
```powershell
npm.cmd run tauri build -- --bundles nsis
```
- **Lokasi Output Installer:**
  `D:\Rust\.nexora-target\release\bundle\nsis\Nexora Finder_<versi>_x64-setup.exe`
- **Salin ke Root Proyek:**
  Salin file installer yang baru selesai di-build ke folder root proyek (`d:\Rust\Nexora Finder\`) agar mudah diakses oleh pengguna.
- **Salin Binary Standalone:**
  Salin `D:\Rust\.nexora-target\release\nexora-finder.exe` ke `d:\Rust\Nexora Finder\nexora-finder.exe` dan restart proses aplikasi yang sedang berjalan.

---

### 3. Commit Perubahan Versi ke Git Lokal
Stage seluruh file konfigurasi dan kode yang telah berubah:
```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json src/components/SettingsModal.tsx
git commit -m "chore(release): bump version to v<versi> and rebuild installer"
```
*(Ingat aturan `RULES_GIT_POLICY.md`: Dilarang `git push` otomatis kecuali user secara eksplisit meminta push!)*

---

### 4. Format Laporan ke Pengguna
Tampilkan ringkasan hasil rilis kepada pengguna dengan struktur:
1. **Versi Baru Aktif:** Nomor versi yang baru diterapkan.
2. **Lokasi File Installer:** Lokasi file `.exe` setup baru di root folder.
3. **Status Binary Running:** Konfirmasi bahwa binary yang sedang jalan di system tray sudah diperbarui.
4. **Instruksi Git Push:** Berikan perintah `git push origin main` (dan git tag jika diperlukan) agar pengguna dapat mengeksekusinya jika siap.

