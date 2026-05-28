import { Request, Response, NextFunction } from 'express';
import { getEvents } from '../services/indexer';
import { AdminEvent, FeeHistoryItem, ApiResponse } from '../types';

/** GET /api/admin/events — returns all indexed contract events */
export async function getAllEvents(req: Request, res: Response, next: NextFunction) {
  try {
    const events = getEvents() as unknown as AdminEvent[];
    const body: ApiResponse<AdminEvent[]> = { success: true, data: events };
    res.json(body);
  } catch (err) {
    next(err);
  }
}

/** GET /api/admin/fees — returns fees_withdrawn event payloads */
export async function getFeeSummary(req: Request, res: Response, next: NextFunction) {
  try {
    const withdrawals = getEvents('fees_withdrawn').map((e) => e.payload as unknown as FeeHistoryItem);
    const body: ApiResponse<FeeHistoryItem[]> = { success: true, data: withdrawals };
    res.json(body);
  } catch (err) {
    next(err);
  }
}
