<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 14:40:54 Europe/London
REASON FOR CREATION: Zapis pomiaru wdrozonego obrazu Darkstar z zywymi, wylacznie odczytowymi kafelkami swiata.
==========================================
-->

# Darkstar — kafelki świata live, 2026-09-04

1. Pomiar CBMS: `2026-09-04T14:40:54+01:00`; host HEAD `3ad0df7df733`, worktree czysty.
2. `docker build --network=host -t darkstar:dev .` zakonczyl sie kodem 0; obraz to `sha256:f0b2473c580f1f0119a7e4d660ab71381f1097bf174b46fc23b9c082e11cc612`.
3. Compose odtworzyl tylko serwis `darkstar`; identyfikatory kontenerow Headplane i Headscale pozostaly bez zmiany.
4. Kontener `darkstar` uzywa zmierzonego SHA obrazu, jest `running/healthy`, a `127.0.0.1:18080/health` zwraca HTTP 200.
5. `ss` pokazuje dwa i tylko dwa listenery 18080: `127.0.0.1:18080` oraz `192.168.2.1:18080`.
6. `/world/` i `/world/world.js` pod `192.168.2.1:18080` zwracaja HTTP 200; HTML laduje `./world.js`, a skrypt czyta `/v1/world/status`.
7. `/v1/world/status` ma `readOnly: true` i zwraca `up` dla Darkstar, Headscale oraz Headplane; brak metod POST/PUT/PATCH/DELETE jest przypiety testem.
8. Headscale `/windows` na 8080 zwraca 200; Headplane slucha tylko na `127.0.0.1:3000` i `/admin` zwraca 302; cloudflared i Tailscale SaaS sa `active`; bez `sudo` i zmian sieci hosta.
