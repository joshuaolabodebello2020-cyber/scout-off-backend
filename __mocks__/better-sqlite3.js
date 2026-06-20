/**
 * Manual Jest mock for better-sqlite3.
 * Provides a minimal in-memory SQL-like interface so tests can run without
 * the native binary (which requires a matching Node ABI).
 */

class Statement {
  constructor(db, sql) {
    this._db = db;
    this._sql = sql.trim();
  }

  run(...args) {
    const sql = this._sql.toUpperCase();
    if (sql.startsWith('INSERT OR IGNORE INTO EVENTS')) {
      const [type, ledger, txHash, payload] = args;
      if (!this._db._events.find((e) => e.tx_hash === txHash)) {
        this._db._events.push({ type, ledger, tx_hash: txHash, payload });
      }
    } else if (sql.startsWith('INSERT INTO INDEXER_STATE') || sql.startsWith('INSERT OR REPLACE INTO INDEXER_STATE')) {
      const [key, value] = args;
      this._db._state.set(key, value);
    } else if (sql.startsWith('INSERT INTO PLAYERS')) {
      const [player_id, wallet, position, region, metadata_uri, created_at] = args;
      const existing = this._db._players.findIndex((p) => p.player_id === player_id);
      if (existing >= 0) {
        // ON CONFLICT DO UPDATE — update mutable fields
        this._db._players[existing] = {
          ...this._db._players[existing],
          wallet,
          position,
          region,
          metadata_uri,
        };
      } else {
        this._db._players.push({ player_id, wallet, position, region, metadata_uri, progress_level: 0, created_at });
      }
    } else if (sql.startsWith('UPDATE PLAYERS SET PROGRESS_LEVEL')) {
      const [level, player_id] = args;
      const idx = this._db._players.findIndex((p) => p.player_id === player_id);
      if (idx >= 0) this._db._players[idx].progress_level = level;
    }
    return { changes: 1, lastInsertRowid: 0 };
  }

  get(...args) {
    const sql = this._sql.toUpperCase();
    if (sql.includes('INDEXER_STATE')) {
      const key = args[0];
      const value = this._db._state.get(key);
      return value !== undefined ? { value } : undefined;
    }
    if (sql.includes('FROM PLAYERS') && sql.includes('WHERE PLAYER_ID = ?')) {
      return this._db._players.find((p) => p.player_id === args[0]) ?? undefined;
    }
    return undefined;
  }

  all(...args) {
    const sql = this._sql.toUpperCase();
    if (sql.includes('FROM EVENTS')) {
      if (sql.includes('WHERE TYPE = ?')) {
        return this._db._events.filter((e) => e.type === args[0]);
      }
      return [...this._db._events];
    }
    if (sql.includes('FROM PLAYERS')) {
      let rows = [...this._db._players];
      // Parse WHERE conditions from remaining args in order
      const whereMatch = sql.match(/WHERE (.+?)(?:ORDER|$)/);
      if (whereMatch) {
        const conditions = whereMatch[1].split(' AND ');
        let argIdx = 0;
        for (const cond of conditions) {
          const val = args[argIdx++];
          if (cond.includes('REGION = ?')) rows = rows.filter((r) => r.region === val);
          else if (cond.includes('POSITION = ?')) rows = rows.filter((r) => r.position === val);
          else if (cond.includes('PROGRESS_LEVEL >= ?')) rows = rows.filter((r) => r.progress_level >= val);
        }
      }
      return rows;
    }
    return [];
  }
}

class Database {
  constructor(_path) {
    this._events = [];
    this._state = new Map();
    this._players = [];
  }

  exec(_sql) {
    // no-op: CREATE TABLE statements are ignored
  }

  prepare(sql) {
    return new Statement(this, sql);
  }

  transaction(fn) {
    return (...args) => fn(...args);
  }

  close() {}
}

module.exports = Database;
