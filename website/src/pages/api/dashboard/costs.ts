import type { NextApiRequest, NextApiResponse } from 'next';

interface CostEntry {
  provider: string;
  requests: number;
  cost: number;
  timestamp: string;
}

function getCostHistory(): CostEntry[] {
  if (typeof window !== 'undefined') return [];
  try {
    const fs = require('fs');
    const path = require('path');
    const costsPath = path.join(process.cwd(), '../../.free-mode-costs.json');
    if (fs.existsSync(costsPath)) {
      const data = JSON.parse(fs.readFileSync(costsPath, 'utf-8'));
      return data.history || [];
    }
  } catch (err) {
    console.error('Failed to load cost history:', err);
  }
  return [];
}

export default function handler(req: NextApiRequest, res: NextApiResponse<CostEntry[] | { error: string }>) {
  if (req.method !== 'GET') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const history = getCostHistory();
    // Return last 100 entries
    res.status(200).json(history.slice(-100));
  } catch (error) {
    console.error('Cost history error:', error);
    res.status(500).json({ error: 'Failed to fetch cost history' });
  }
}
