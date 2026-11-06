# RULES_GIT_POLICY.md — Aturan Baku Git Push (Wajib Dipatuhi AI)

> **PERINGATAN UNTUK AI CODING ASSISTANT:**
> Dokumen ini memuat aturan **ABSOLUT** terkait pengelolaan Git di repositori Nexora Finder. Pelanggaran terhadap aturan ini dilarang keras.

---

## 🚫 ATURAN UTAMA: JANGAN PERNAH `git push` SECARA OTOMATIS!

1. **Dilarang Menjalankan `git push` Tanpa Perintah Eksplisit:**
   - Setiap kali Anda selesai mengubah kode, memperbaiki bug, mendesain UI, atau melakukan build, **HANYA LAKUKAN `git commit` DI REPOSITORI LOKAL**.
   - **JANGAN PERNAH** mengeksekusi `git push`, `git push origin main`, atau perintah push remote lainnya secara mandiri.
   - Perintah `git push` di Windows sering kali memicu dialog interaktif *Git Credential Manager* yang menyebabkan proses terminal AI macet (hang).

2. **Tunggu Perintah Spesifik dari Pengguna (User):**
   - Hanya jalankan atau sarankan `git push` apabila pengguna **secara eksplisit** mengetik:
     - *"push ke git"*
     - *"upload ke github"*
     - *"tolong push sekarang"*
   - Jika pengguna tidak meminta push, cukup beri tahu pengguna bahwa commit lokal telah selesai dan berikan instruksi bahwa mereka dapat menjalankan `git push origin main` sendiri di terminal jika sudah siap.

3. **Format Konfirmasi Commit yang Benar:**
   - Setelah commit lokal selesai, sertakan pesan penutup seperti berikut:
     ```markdown
     > [!TIP]
     > Perubahan telah di-commit ke Git lokal (Hash: `xxxxxxx`). 
     > Jika Anda ingin mempublikasikannya ke GitHub sekarang, jalankan perintah berikut atau beri tahu saya:
     > ```powershell
     > git push origin main
     > ```
     ```
