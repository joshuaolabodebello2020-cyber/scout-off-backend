// ─── Player ──────────────────────────────────────────────────────────────────

export type ProgressLevel = 0 | 1 | 2 | 3;

export interface Player {
  playerId: string;
  wallet: string;
  metadataUri: string; // IPFS CID pointing to off-chain profile JSON
  position: string;
  region: string;
  progressLevel: ProgressLevel;
  createdAt: number; // ledger timestamp
}

export interface PlayerMetadata {
  name: string;
  age: number;
  position: string;
  region: string;
  club?: string;
  highlightReels: string[]; // IPFS CIDs
  stats?: Record<string, string | number>;
}

// ─── Milestone ────────────────────────────────────────────────────────────────

export type MilestoneType = 'identity' | 'performance' | 'trial_offer';

export interface Milestone {
  milestoneId: string;
  playerId: string;
  milestoneType: MilestoneType;
  evidenceUri: string; // IPFS CID
  validator: string;   // Stellar address
  approved: boolean;
  createdAt: number;
}

// ─── Scout ────────────────────────────────────────────────────────────────────

export interface Scout {
  wallet: string;
  subscriptionExpiry?: number; // ledger timestamp; undefined = no active sub
}

export interface ContactUnlock {
  scout: string;
  playerId: string;
  unlockedAt: number;
}

// ─── API shapes ───────────────────────────────────────────────────────────────

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  total: number;
  page: number;
  pageSize: number;
}

export interface FilterPlayersQuery {
  region?: string;
  position?: string;
  minTier?: ProgressLevel;
  page?: number;
  pageSize?: number;
}

// ─── SEP-10 ───────────────────────────────────────────────────────────────────

export interface Sep10Challenge {
  challenge: string; // XDR
  networkPassphrase: string;
}

export interface Sep10Token {
  token: string;
  account: string;
  expiresAt: number; // Unix timestamp
}

// ─── Contract events (indexed) ────────────────────────────────────────────────

export type ContractEventType =
  | 'player_registered'
  | 'milestone_submitted'
  | 'milestone_approved'
  | 'scout_subscribed'
  | 'contact_unlocked'
  | 'trial_offer_logged'
  | 'fees_withdrawn';

export interface ContractEvent {
  type: ContractEventType;
  ledger: number;
  txHash: string;
  payload: Record<string, unknown>;
}

// ─── Admin event payload types ────────────────────────────────────────────────

export interface PlayerRegisteredPayload {
  eventType: 'player_registered';
  player_id: string;
  wallet: string;
  metadata_uri: string;
  position: string;
  region: string;
  timestamp: number;
}

export interface MilestoneSubmittedPayload {
  eventType: 'milestone_submitted';
  milestone_id: string;
  player_id: string;
  milestone_type: MilestoneType;
  evidence_uri: string;
  validator: string;
  timestamp: number;
}

export interface MilestoneApprovedPayload {
  eventType: 'milestone_approved';
  milestone_id: string;
  player_id: string;
  milestone_type: MilestoneType;
  validator: string;
  timestamp: number;
}

export interface ScoutSubscribedPayload {
  eventType: 'scout_subscribed';
  wallet: string;
  subscription_expiry: number;
  timestamp: number;
}

export interface ContactUnlockedPayload {
  eventType: 'contact_unlocked';
  scout: string;
  player_id: string;
  unlocked_at: number;
  timestamp: number;
}

export interface TrialOfferLoggedPayload {
  eventType: 'trial_offer_logged';
  player_id: string;
  scout: string;
  timestamp: number;
}

/** Payload emitted when platform fees are withdrawn. */
export interface FeesWithdrawnPayload {
  eventType: 'fees_withdrawn';
  amount: string;
  token: string;
  destination: string;
  timestamp: number;
}

/**
 * Discriminated union over all contract event payload shapes.
 * Use `payload.eventType` to narrow to a specific variant.
 */
export type ContractEventPayload =
  | PlayerRegisteredPayload
  | MilestoneSubmittedPayload
  | MilestoneApprovedPayload
  | ScoutSubscribedPayload
  | ContactUnlockedPayload
  | TrialOfferLoggedPayload
  | FeesWithdrawnPayload;

/** Fully-typed contract event used in admin API responses. */
export interface AdminEvent {
  type: ContractEventType;
  ledger: number;
  txHash: string;
  payload: ContractEventPayload;
}

/** Fee history item returned by GET /api/admin/fees. */
export interface FeeHistoryItem {
  amount: string;
  token: string;
  destination: string;
  timestamp: number;
  eventType: 'fees_withdrawn';
}
