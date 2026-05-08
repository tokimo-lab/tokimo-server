#!/usr/bin/env node
// Mixed cache-hit / cache-miss load tester for tokimo-server.
//
// Hits a representative GET endpoint per provider with a mix of:
//   - `repeat` requests that reuse a small pool of params (cache hits expected
//     after the first miss)
//   - `unique` requests that randomize a param so each one is a fresh miss
//
// Defaults: 500 total requests, ~70% repeat / 30% unique, 16 concurrent workers.
//
// Usage:
//   node tests/load-test/load-test.mjs                 # local server :5680
//   BASE_URL=http://localhost:5680 \
//   SERVICE_KEY=tks_xxx \
//   TOTAL=500 CONCURRENCY=16 REPEAT_RATIO=0.7 \
//     node tests/load-test/load-test.mjs

const BASE_URL = process.env.BASE_URL ?? "http://localhost:5680";
const SERVICE_KEY =
  process.env.SERVICE_KEY ?? "tks_0qcWQe7cIigPMxnpHfojnVjOAZcMwuU8";
const TOTAL = Number(process.env.TOTAL ?? 500);
const CONCURRENCY = Number(process.env.CONCURRENCY ?? 16);
const REPEAT_RATIO = Number(process.env.REPEAT_RATIO ?? 0.7);
const TIMEOUT_MS = Number(process.env.TIMEOUT_MS ?? 15000);

// ---------------------------------------------------------------------------
// Param pools. `repeat` is a small fixed list (drawn round-robin to ensure the
// first hit warms cache). `unique` is a generator that returns a fresh URL
// every call so it always misses.
// ---------------------------------------------------------------------------

const pick = (arr) => arr[Math.floor(Math.random() * arr.length)];
const rand = () => Math.random().toString(36).slice(2, 10);

/** @type {Array<{name:string, repeat:string[], unique:()=>string}>} */
const ENDPOINTS = [
  {
    name: "tmdb.movie",
    repeat: ["/api/tmdb/movie/550", "/api/tmdb/movie/680", "/api/tmdb/movie/13"],
    unique: () => `/api/tmdb/movie/${1 + Math.floor(Math.random() * 200000)}`,
  },
  {
    name: "tmdb.tv",
    repeat: ["/api/tmdb/tv/1399", "/api/tmdb/tv/1396"],
    unique: () => `/api/tmdb/tv/${1 + Math.floor(Math.random() * 100000)}`,
  },
  {
    name: "omdb.title",
    repeat: ["/api/omdb/title/tt0111161", "/api/omdb/title/tt0068646"],
    unique: () =>
      `/api/omdb/title/tt${String(Math.floor(Math.random() * 9999999)).padStart(7, "0")}`,
  },
  {
    name: "thetvdb.series",
    repeat: ["/api/thetvdb/series/121361", "/api/thetvdb/series/81189"],
    unique: () =>
      `/api/thetvdb/series/${1 + Math.floor(Math.random() * 500000)}`,
  },
  {
    name: "bangumi.subject",
    repeat: ["/api/bangumi/subject/1", "/api/bangumi/subject/253"],
    unique: () =>
      `/api/bangumi/subject/${1 + Math.floor(Math.random() * 400000)}`,
  },
  {
    name: "bangumi.search",
    repeat: ["/api/bangumi/search?keyword=fate", "/api/bangumi/search?keyword=clannad"],
    unique: () => `/api/bangumi/search?keyword=${rand()}`,
  },
  {
    name: "douban.search",
    repeat: ["/api/douban/search?q=肖申克", "/api/douban/search?q=教父"],
    unique: () => `/api/douban/search?q=${encodeURIComponent(rand())}`,
  },
  {
    name: "fanart.movie",
    repeat: ["/api/fanart/movie/550", "/api/fanart/movie/680"],
    unique: () => `/api/fanart/movie/${1 + Math.floor(Math.random() * 200000)}`,
  },
  {
    name: "spotify.search",
    repeat: [
      "/api/spotify/search?q=coldplay&type=artist",
      "/api/spotify/search?q=radiohead&type=artist",
    ],
    unique: () => `/api/spotify/search?q=${rand()}&type=artist`,
  },
  {
    name: "musicbrainz.search",
    repeat: [
      "/api/musicbrainz/search?type=artist&q=beatles",
      "/api/musicbrainz/search?type=artist&q=queen",
    ],
    unique: () => `/api/musicbrainz/search?type=artist&q=${rand()}`,
  },
  {
    name: "deezer.search",
    repeat: ["/api/deezer/search?q=daft%20punk", "/api/deezer/search?q=adele"],
    unique: () => `/api/deezer/search?q=${rand()}`,
  },
  {
    name: "lrclib.get",
    repeat: [
      "/api/lrclib/get?artist_name=Coldplay&track_name=Yellow",
      "/api/lrclib/get?artist_name=Adele&track_name=Hello",
    ],
    unique: () =>
      `/api/lrclib/get?artist_name=${rand()}&track_name=${rand()}`,
  },
  {
    name: "wikipedia.summary",
    repeat: [
      "/api/wikipedia/summary?title=Rust_(programming_language)&lang=en",
      "/api/wikipedia/summary?title=TypeScript&lang=en",
    ],
    unique: () =>
      `/api/wikipedia/summary?title=${rand()}&lang=en`,
  },
  {
    name: "qidian.search",
    repeat: ["/api/qidian/search?q=斗罗", "/api/qidian/search?q=诡秘"],
    unique: () => `/api/qidian/search?q=${encodeURIComponent(rand())}`,
  },
  {
    name: "openmeteo.forecast",
    repeat: [
      "/api/openmeteo/forecast?latitude=39.9&longitude=116.4",
      "/api/openmeteo/forecast?latitude=31.2&longitude=121.5",
    ],
    unique: () => {
      const lat = (Math.random() * 180 - 90).toFixed(4);
      const lon = (Math.random() * 360 - 180).toFixed(4);
      return `/api/openmeteo/forecast?latitude=${lat}&longitude=${lon}`;
    },
  },
  {
    name: "nominatim.search",
    repeat: ["/api/nominatim/search?q=Beijing", "/api/nominatim/search?q=Tokyo"],
    unique: () => `/api/nominatim/search?q=${rand()}`,
  },
  {
    name: "geocoding.forward",
    repeat: [
      "/api/geocoding/forward?q=Shanghai",
      "/api/geocoding/forward?q=Paris",
    ],
    unique: () => `/api/geocoding/forward?q=${rand()}`,
  },
  {
    name: "holiday",
    repeat: ["/api/holiday/CN/2024", "/api/holiday/US/2024"],
    unique: () => {
      const year = 2000 + Math.floor(Math.random() * 50);
      const cc = pick(["CN", "US", "JP", "DE", "FR", "GB", "KR", "IN"]);
      return `/api/holiday/${cc}/${year}`;
    },
  },
  {
    name: "github.releases.latest",
    repeat: [
      "/api/github/releases/rust-lang/rust/latest",
      "/api/github/releases/microsoft/typescript/latest",
    ],
    unique: () =>
      `/api/github/releases/${rand()}/${rand()}/latest`,
  },
  {
    name: "currency.rates",
    repeat: [
      "/api/currency/rates?base=USD&targets=CNY,EUR,JPY&days=7",
      "/api/currency/rates?base=EUR&targets=USD,GBP&days=7",
    ],
    unique: () => {
      const bases = ["USD", "EUR", "GBP", "JPY", "CNY", "AUD", "CAD"];
      const base = pick(bases);
      const targets = bases.filter((c) => c !== base).slice(0, 3).join(",");
      const days = 1 + Math.floor(Math.random() * 30);
      return `/api/currency/rates?base=${base}&targets=${targets}&days=${days}`;
    },
  },
  {
    name: "assrt.search",
    repeat: ["/api/assrt/search?q=Inception", "/api/assrt/search?q=Frozen"],
    unique: () => `/api/assrt/search?q=${rand()}`,
  },
  {
    name: "hot.list",
    repeat: ["/api/hot/list?source=baidu", "/api/hot/list?source=weibo"],
    unique: () => `/api/hot/list?source=${rand()}`,
  },
  {
    name: "sports.schedule",
    repeat: ["/api/sports/schedule?date=2025-01-01", "/api/sports/schedule?date=2025-02-01"],
    unique: () => {
      const m = 1 + Math.floor(Math.random() * 12);
      const d = 1 + Math.floor(Math.random() * 28);
      return `/api/sports/schedule?date=2025-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    },
  },
  {
    name: "hitokoto.sentence",
    repeat: ["/api/hitokoto/sentence?c=a", "/api/hitokoto/sentence?c=d", "/api/hitokoto/sentence"],
    unique: () => {
      const cats = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"];
      return `/api/hitokoto/sentence?c=${pick(cats)}`;
    },
  },
  {
    name: "zenquotes.random",
    repeat: ["/api/zenquotes/random"],
    unique: () => "/api/zenquotes/random",
  },
  {
    name: "bing.wallpaper",
    repeat: [
      "/api/bing/wallpaper?mkt=zh-CN&n=1&idx=0",
      "/api/bing/wallpaper?mkt=en-US&n=1&idx=0",
      "/api/bing/wallpaper?mkt=ja-JP&n=4&idx=0",
    ],
    unique: () => {
      const mkts = ["zh-CN", "en-US", "ja-JP"];
      const idx = Math.floor(Math.random() * 8);
      const n = 1 + Math.floor(Math.random() * 4);
      return `/api/bing/wallpaper?mkt=${pick(mkts)}&n=${n}&idx=${idx}`;
    },
  },
  {
    name: "opensubtitles.search",
    repeat: [
      "/api/opensubtitles/search?imdb_id=tt1375666&languages=en",
      "/api/opensubtitles/search?query=Inception&languages=en,zh-cn",
    ],
    unique: () =>
      `/api/opensubtitles/search?imdb_id=tt${String(Math.floor(Math.random() * 9999999)).padStart(7, "0")}&languages=en`,
  },
  {
    name: "regielive.search",
    repeat: ["/api/regielive/search?nume=Inception", "/api/regielive/search?nume=Frozen"],
    unique: () => `/api/regielive/search?nume=${rand()}`,
  },
  {
    name: "gestdown.shows.search",
    repeat: [
      "/api/gestdown/shows/search?title=Game%20of%20Thrones",
      "/api/gestdown/shows/search?title=Friends",
    ],
    unique: () => `/api/gestdown/shows/search?title=${rand()}`,
  },
  {
    name: "gestdown.subtitles",
    repeat: [
      "/api/gestdown/subtitles?show_id=4d6f3c4a-37b3-43e1-8f9e-1f1f1f1f1f1f&season=1&episode=1&lang=English",
    ],
    unique: () => {
      const s = 1 + Math.floor(Math.random() * 10);
      const e = 1 + Math.floor(Math.random() * 24);
      return `/api/gestdown/subtitles?show_id=${rand()}&season=${s}&episode=${e}&lang=English`;
    },
  },
];

// ---------------------------------------------------------------------------
// Plan
// ---------------------------------------------------------------------------

/** Build the request plan: one URL per slot. */
function plan(total) {
  const out = [];
  for (let i = 0; i < total; i++) {
    const ep = ENDPOINTS[i % ENDPOINTS.length];
    const isRepeat = Math.random() < REPEAT_RATIO;
    const url = isRepeat ? pick(ep.repeat) : ep.unique();
    out.push({ name: ep.name, mode: isRepeat ? "repeat" : "unique", url });
  }
  // shuffle so endpoints are interleaved
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

const stats = new Map(); // name -> {total, ok, err, repeat, unique, durations[]}

function record(name, mode, ok, durMs) {
  let s = stats.get(name);
  if (!s) {
    s = { total: 0, ok: 0, err: 0, repeat: 0, unique: 0, durations: [] };
    stats.set(name, s);
  }
  s.total++;
  if (ok) s.ok++;
  else s.err++;
  if (mode === "repeat") s.repeat++;
  else s.unique++;
  s.durations.push(durMs);
}

async function fire(req) {
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), TIMEOUT_MS);
  const start = performance.now();
  try {
    const res = await fetch(BASE_URL + req.url, {
      headers: { Authorization: `Bearer ${SERVICE_KEY}` },
      signal: ctrl.signal,
    });
    // drain body (small) so server can close & cache write completes
    await res.text();
    const dur = performance.now() - start;
    record(req.name, req.mode, res.ok, dur);
    return { ok: res.ok, status: res.status, dur };
  } catch (e) {
    const dur = performance.now() - start;
    record(req.name, req.mode, false, dur);
    return { ok: false, status: 0, dur, err: String(e) };
  } finally {
    clearTimeout(t);
  }
}

async function worker(queue) {
  while (queue.length) {
    const req = queue.shift();
    if (!req) break;
    await fire(req);
  }
}

function pct(arr, p) {
  if (!arr.length) return 0;
  const sorted = [...arr].sort((a, b) => a - b);
  const idx = Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length));
  return sorted[idx];
}

async function main() {
  console.log(
    `[load-test] base=${BASE_URL} total=${TOTAL} concurrency=${CONCURRENCY} repeat_ratio=${REPEAT_RATIO}`,
  );
  const queue = plan(TOTAL);
  const totalStart = performance.now();
  let lastTick = totalStart;
  let done = 0;

  const tickInterval = setInterval(() => {
    const elapsed = (performance.now() - totalStart) / 1000;
    const rps = done / Math.max(elapsed, 0.001);
    process.stdout.write(
      `\r[progress] ${done}/${TOTAL}  ${rps.toFixed(1)} req/s  elapsed=${elapsed.toFixed(1)}s`,
    );
  }, 500);

  const wrappedFire = async (req) => {
    await fire(req);
    done++;
  };
  const localWorker = async () => {
    while (queue.length) {
      const req = queue.shift();
      if (!req) break;
      await wrappedFire(req);
    }
  };

  const workers = Array.from({ length: CONCURRENCY }, () => localWorker());
  await Promise.all(workers);
  clearInterval(tickInterval);

  const totalDur = (performance.now() - totalStart) / 1000;
  process.stdout.write("\n");

  // ---------- summary ----------
  const rows = [];
  let grandTotal = 0,
    grandOk = 0,
    grandErr = 0,
    grandRepeat = 0,
    grandUnique = 0;
  const allDur = [];

  for (const [name, s] of [...stats.entries()].sort()) {
    grandTotal += s.total;
    grandOk += s.ok;
    grandErr += s.err;
    grandRepeat += s.repeat;
    grandUnique += s.unique;
    allDur.push(...s.durations);
    rows.push({
      provider: name,
      total: s.total,
      ok: s.ok,
      err: s.err,
      repeat: s.repeat,
      unique: s.unique,
      p50_ms: Math.round(pct(s.durations, 50)),
      p95_ms: Math.round(pct(s.durations, 95)),
    });
  }

  console.log("\n=== per-endpoint ===");
  console.table(rows);

  console.log("\n=== overall ===");
  console.log({
    total: grandTotal,
    ok: grandOk,
    err: grandErr,
    repeat: grandRepeat,
    unique: grandUnique,
    duration_s: Number(totalDur.toFixed(2)),
    rps: Number((grandTotal / totalDur).toFixed(1)),
    p50_ms: Math.round(pct(allDur, 50)),
    p95_ms: Math.round(pct(allDur, 95)),
    p99_ms: Math.round(pct(allDur, 99)),
  });

  console.log(
    "\nTip: now open the admin Dashboard to inspect calls/errors/cache-hit charts.",
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
