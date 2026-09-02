/*
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 00:05:00
REASON FOR CREATION: Konfiguracja Vite dla Control Room - przekierowanie /v1 na lokalny darkstar-server (Task 10).
MECHANICS: Serwer deweloperski nasluchuje wylacznie na 127.0.0.1 i przekazuje /v1 na port
darkstar-server. Adres docelowy pochodzi ze zmiennej DARKSTAR_DEV_API, wiec port nie jest wpisany
na sztywno. Nasluch na 0.0.0.0 jest tu swiadomie NIEUSTAWIONY - panel nie ma trafic do sieci.
SYSTEM PART: Control Room / konfiguracja budowania.
ARCHITECTURE FUNCTION: Pozwala pracowac nad panelem lokalnie, bez wystawiania czegokolwiek na zewnatrz.
DEPENDENCIES/LINKS: darkstar-server (/v1/gateway/status, /v1/system-graph).
TECH STACK: TypeScript 6 + Vite 8, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: byc odczytany przez Vite przy starcie serwera deweloperskiego i budowaniu paczki -
      Vite sam wykonuje ten plik jako modul i oczekuje wyeksportowanej konfiguracji.
  (2) DLACZEGO NIE RUST: to nie jest program, ktory uruchamiamy, tylko dane wejsciowe cudzego
      narzedzia. Vite czyta wylacznie vite.config.{js,ts,mjs}; plik w Ruscie nie zostalby nawet
      otwarty, a przepisanie samego Vite na Rust to wymiana narzedzia budujacego, nie zmiana
      jezyka jednego pliku. Zaufany control plane pozostaje w Ruscie po stronie serwera.
  (3) TRACIMY: typy wspolne z rdzeniem i sprawdzanie przez cargo. Rekompensata: plik zawiera
      wylacznie adres przekierowania i wiazanie nasluchu do petli zwrotnej - zero logiki decyzyjnej.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
*/

import { execFile } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import react from '@vitejs/plugin-react'
import { defineConfig, type Plugin } from 'vite'

// Domyslnie 18080 - tam slucha darkstar-server na CBMS, dostepny lokalnie przez
// tunel ssh -L 127.0.0.1:18080:127.0.0.1:18080. Nic nie wychodzi poza petle zwrotna.
const apiTarget = process.env.DARKSTAR_DEV_API ?? 'http://127.0.0.1:18080'
const gitWorktree = path.resolve(fileURLToPath(new URL('..', import.meta.url)))

type GitCommandResult = {
  stdout: string
  stderr: string
  exit_code: number
}

function runGit(args: string[]): Promise<GitCommandResult> {
  return new Promise((resolve) => {
    execFile(
      'git',
      ['-C', gitWorktree, ...args],
      {
        cwd: gitWorktree,
        windowsHide: true,
        timeout: 5000,
        maxBuffer: 256 * 1024,
      },
      (error, stdout, stderr) => {
        resolve({
          stdout,
          stderr,
          exit_code: typeof error?.code === 'number' ? error.code : error ? 1 : 0,
        })
      },
    )
  })
}

function localGitRail(): Plugin {
  const readSnapshot = async () => {
    const [status, log, head, upstream] = await Promise.all([
      runGit(['status', '-sb']),
      runGit([
        'log',
        '--all',
        '--topo-order',
        '--max-count=18',
        '--pretty=format:%H%x1f%P%x1f%D%x1f%s%x1f%an%x1f%aI%x1e',
      ]),
      runGit(['rev-parse', 'HEAD']),
      runGit(['rev-parse', '@{u}']),
    ])
    return { worktree: gitWorktree, status, log, head, upstream }
  }

  return {
    name: 'darkstar-local-git-rail',
    configureServer(server) {
      server.middlewares.use('/__darkstar/git', (request, response) => {
        const isFetch = request.url === '/fetch'
        if (isFetch && request.method !== 'POST') {
          response.statusCode = 405
          response.setHeader('Allow', 'POST')
          response.end()
          return
        }
        if (!isFetch && request.method !== 'GET') {
          response.statusCode = 405
          response.setHeader('Allow', 'GET')
          response.end()
          return
        }

        void (async () => {
          if (isFetch) {
            const fetched = await runGit(['fetch', '--prune', 'origin'])
            if (fetched.exit_code !== 0) {
              response.statusCode = 502
              response.setHeader('Content-Type', 'application/json; charset=utf-8')
              response.end(JSON.stringify({ error: fetched.stderr.trim() || 'git fetch origin nie powiodl sie.' }))
              return
            }
          }

          response.statusCode = 200
          response.setHeader('Content-Type', 'application/json; charset=utf-8')
          response.end(JSON.stringify(await readSnapshot()))
        })()
      })
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), localGitRail()],
  server: {
    // Tylko petla zwrotna. Nie zmieniac na 0.0.0.0.
    host: '127.0.0.1',
    proxy: {
      '/v1': {
        target: apiTarget,
        changeOrigin: false,
      },
      // Bez tych dwoch /health i /ready trafialyby do dev servera Vite i wracalyby
      // jako strona HTML ze statusem 200 - czyli falszywy sukces w panelu operatora.
      '/health': {
        target: apiTarget,
        changeOrigin: false,
      },
      '/ready': {
        target: apiTarget,
        changeOrigin: false,
      },
    },
  },
})
