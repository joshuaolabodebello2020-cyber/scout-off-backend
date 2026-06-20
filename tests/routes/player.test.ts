import request from 'supertest';
import jwt from 'jsonwebtoken';
import app from '../../src/app';

const SECRET = process.env.JWT_SECRET ?? 'test-secret';

jest.mock('../../src/db', () => ({
  getEvents: jest.fn().mockReturnValue([]),
  queryPlayers: jest.fn().mockReturnValue([]),
  getPlayerById: jest.fn().mockReturnValue(null),
}));

jest.mock('../../src/services/indexer', () => ({
  indexEvents: jest.fn(),
  normalizeEventId: jest.fn(),
}));

jest.mock('../../src/services/ipfs', () => ({
  pinJson: jest.fn().mockResolvedValue('QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG'),
  gatewayUrl: jest.fn((cid: string) => `https://gateway.pinata.cloud/ipfs/${cid}`),
}));

jest.mock('../../src/services/webhooks', () => ({
  dispatchEventWebhook: jest.fn().mockResolvedValue(undefined),
}));

jest.mock('../../src/services/cache', () => ({
  invalidatePlayerCache: jest.fn(),
}));

function makeToken(wallet: string, role: string): string {
  return jwt.sign({ sub: wallet, role }, SECRET, { expiresIn: '1h' });
}

const PLAYER_WALLET = 'G' + 'A'.repeat(55);

const validPayload = {
  wallet: PLAYER_WALLET,
  position: 'striker',
  region: 'europe',
  metadataUri: 'QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG',
};

// ─── POST /api/players/register ───────────────────────────────────────────────

describe('POST /api/players/register — role enforcement', () => {
  it('returns 401 when no token provided', async () => {
    const res = await request(app)
      .post('/api/players/register')
      .send(validPayload);
    expect(res.status).toBe(401);
    expect(res.body.success).toBe(false);
  });

  it('returns 403 when validator JWT provided', async () => {
    const token = makeToken(PLAYER_WALLET, 'validator');
    const res = await request(app)
      .post('/api/players/register')
      .set('Authorization', `Bearer ${token}`)
      .send(validPayload);
    expect(res.status).toBe(403);
    expect(res.body.success).toBe(false);
  });

  it('returns 403 when scout JWT provided', async () => {
    const token = makeToken(PLAYER_WALLET, 'scout');
    const res = await request(app)
      .post('/api/players/register')
      .set('Authorization', `Bearer ${token}`)
      .send(validPayload);
    expect(res.status).toBe(403);
    expect(res.body.success).toBe(false);
  });

  it('returns 201 when player JWT provided with valid payload', async () => {
    const token = makeToken(PLAYER_WALLET, 'player');
    const res = await request(app)
      .post('/api/players/register')
      .set('Authorization', `Bearer ${token}`)
      .send(validPayload);
    expect(res.status).toBe(201);
    expect(res.body.success).toBe(true);
  });
});

// ─── PUT /api/players/:playerId ───────────────────────────────────────────────

describe('PUT /api/players/:playerId — role enforcement', () => {
  it('returns 401 when no token provided', async () => {
    const res = await request(app)
      .put(`/api/players/${PLAYER_WALLET}`)
      .send({ position: 'midfielder' });
    expect(res.status).toBe(401);
    expect(res.body.success).toBe(false);
  });

  it('returns 403 when validator JWT provided', async () => {
    const token = makeToken(PLAYER_WALLET, 'validator');
    const res = await request(app)
      .put(`/api/players/${PLAYER_WALLET}`)
      .set('Authorization', `Bearer ${token}`)
      .send({ position: 'midfielder' });
    expect(res.status).toBe(403);
    expect(res.body.success).toBe(false);
  });

  it('returns 403 when scout JWT provided', async () => {
    const token = makeToken(PLAYER_WALLET, 'scout');
    const res = await request(app)
      .put(`/api/players/${PLAYER_WALLET}`)
      .set('Authorization', `Bearer ${token}`)
      .send({ position: 'midfielder' });
    expect(res.status).toBe(403);
    expect(res.body.success).toBe(false);
  });

  it('returns 202 when player JWT matches playerId', async () => {
    const token = makeToken(PLAYER_WALLET, 'player');
    const res = await request(app)
      .put(`/api/players/${PLAYER_WALLET}`)
      .set('Authorization', `Bearer ${token}`)
      .send({ position: 'midfielder' });
    expect(res.status).toBe(202);
    expect(res.body.success).toBe(true);
  });
});
