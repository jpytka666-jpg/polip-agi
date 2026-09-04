<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-09-04 14:19:39 Europe/London
REASON FOR CREATION: Zapis odczytowego pomiaru aktywacji unitow i polityk restartu kontenerow konczacej sie fazy Darkstar.
==========================================
-->

# Darkstar — persistence checklist, 2026-09-04

1. Pomiar CBMS: `2026-09-04T14:19:39+01:00`, branch `docs/darkstar-headscale-hotspot-plan`, host HEAD `c04a16151ed7`; wykonano tylko odczyty.
2. `cloudflared.service`: `loaded`, `enabled`, `active`; named tunnel nie byl zatrzymywany ani zmieniany.
3. `darkstar.service`: `loaded`, `enabled`, `active`; kontener `darkstar` jest `running`, `healthy`, restart `on-failure`.
4. `darkstar-headscale.service`: `not-found`; kontener `darkstar-headscale` jest `running`, restart `unless-stopped`, lecz Docker health ma stan `unhealthy`.
5. Mimo Docker health `unhealthy`, zmierzony `http://192.168.2.1:8080/windows` zwraca `200`; nie przypisano temu wyniku statusu healthy.
6. `tailscaled-headscale.service`: `loaded`, `enabled`, `active`; prywatny klient Headscale ma systemd persistence.
7. `darkstar-headplane.service`: `not-found`; kontener `darkstar-headplane` jest `running`, `healthy`, restart `unless-stopped`, a listen to tylko `127.0.0.1:3000`.
8. `http://192.168.2.1:18080/world/` zwraca `200`; listen Darkstar to `127.0.0.1:18080` i `192.168.2.1:18080`; bez rebootu, `sudo` i zmian sieci.
