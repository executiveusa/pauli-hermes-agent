// File: /mnt/skills/user/token-optimization-precoding/token-tracker.ts
// Real-time token and cost tracking implementation

export interface TokenCall {
  timestamp: Date;
  model: string;
  inputTokens: number;
  outputTokens: number;
  cost: number;
  timeMs: number;
  compressed: boolean;
}

export interface TokenMetrics {
  // Estimates vs Actual
  estimatedInputTokens: number;
  actualInputTokens: number;
  estimatedOutputTokens: number;
  actualOutputTokens: number;
  
  // Totals
  totalEstimated: number;
  totalActual: number;
  tokensSaved: number;
  tokensSavedPercent: number;
  
  // Accuracy
  estimationAccuracy: number;  // 0-1 (how close estimate was)
  
  // Costs
  estimatedCost: number;
  actualCost: number;
  costDelta: number;  // Positive = saved, negative = over
  costDeltaPercent: number;
  
  // Metrics
  calls: number;
  averageTokensPerCall: number;
  averageCostPerCall: number;
  totalTime: number;
  averageTimePerCall: number;
}

export class TokenTracker {
  private model: string;
  private estimatedInput: number;
  private estimatedOutput: number;
  private costPerMillion: number;
  
  private callLog: TokenCall[] = [];
  private startTime: Date;
  
  private metrics: TokenMetrics;
  
  constructor(
    model: string,
    estimatedInputTokens: number,
    estimatedOutputTokens: number,
    costPerMillion: number  // e.g., 3.0 for Claude Sonnet
  ) {
    this.model = model;
    this.estimatedInput = estimatedInputTokens;
    this.estimatedOutput = estimatedOutputTokens;
    this.costPerMillion = costPerMillion;
    this.startTime = new Date();
    
    this.metrics = this.initializeMetrics();
  }
  
  private initializeMetrics(): TokenMetrics {
    const totalEstimated = this.estimatedInput + this.estimatedOutput;
    const estimatedCost = totalEstimated * (this.costPerMillion / 1_000_000);
    
    return {
      estimatedInputTokens: this.estimatedInput,
      actualInputTokens: 0,
      estimatedOutputTokens: this.estimatedOutput,
      actualOutputTokens: 0,
      
      totalEstimated,
      totalActual: 0,
      tokensSaved: 0,
      tokensSavedPercent: 0,
      
      estimationAccuracy: 0,
      
      estimatedCost,
      actualCost: 0,
      costDelta: estimatedCost,  // Initially all savings (no API calls yet)
      costDeltaPercent: 100,
      
      calls: 0,
      averageTokensPerCall: 0,
      averageCostPerCall: 0,
      totalTime: 0,
      averageTimePerCall: 0
    };
  }
  
  /**
   * Track an actual API call
   */
  public trackCall(
    inputTokens: number,
    outputTokens: number,
    timeMs: number,
    wasCompressed: boolean = false
  ): void {
    const cost = (inputTokens + outputTokens) * (this.costPerMillion / 1_000_000);
    
    const call: TokenCall = {
      timestamp: new Date(),
      model: this.model,
      inputTokens,
      outputTokens,
      cost,
      timeMs,
      compressed: wasCompressed
    };
    
    this.callLog.push(call);
    
    // Update aggregates
    this.metrics.actualInputTokens += inputTokens;
    this.metrics.actualOutputTokens += outputTokens;
    this.metrics.totalActual += inputTokens + outputTokens;
    this.metrics.actualCost += cost;
    this.metrics.calls += 1;
    this.metrics.totalTime += timeMs;
    
    // Recalculate derived metrics
    this.recalculateMetrics();
    
    // Print update every 5 calls
    if (this.metrics.calls % 5 === 0) {
      this.printRealTimeUpdate();
    }
  }
  
  private recalculateMetrics(): void {
    // Average per call
    if (this.metrics.calls > 0) {
      this.metrics.averageTokensPerCall = 
        this.metrics.totalActual / this.metrics.calls;
      this.metrics.averageCostPerCall = 
        this.metrics.actualCost / this.metrics.calls;
      this.metrics.averageTimePerCall = 
        this.metrics.totalTime / this.metrics.calls;
    }
    
    // Tokens saved
    this.metrics.tokensSaved = 
      this.metrics.totalEstimated - this.metrics.totalActual;
    this.metrics.tokensSavedPercent = 
      this.metrics.totalEstimated > 0
        ? (this.metrics.tokensSaved / this.metrics.totalEstimated) * 100
        : 0;
    
    // Cost delta
    this.metrics.costDelta = 
      this.metrics.estimatedCost - this.metrics.actualCost;
    this.metrics.costDeltaPercent = 
      this.metrics.estimatedCost > 0
        ? (this.metrics.costDelta / this.metrics.estimatedCost) * 100
        : 0;
    
    // Estimation accuracy
    if (this.metrics.totalActual > 0) {
      const ratio = this.metrics.totalEstimated / this.metrics.totalActual;
      this.metrics.estimationAccuracy = 
        ratio <= 1 ? ratio : (1 / ratio);  // Always 0-1
    }
  }
  
  /**
   * Print real-time update (shown during work)
   */
  private printRealTimeUpdate(): void {
    console.log(`
    ┌─────────────────────────────────────────────────────┐
    │ 📊 REAL-TIME TOKEN TRACKING (Call #${this.metrics.calls})          │
    ├─────────────────────────────────────────────────────┤
    │                                                       │
    │ TOKENS:                                             │
    │   Estimated:    ${this.padRight(this.metrics.totalEstimated.toLocaleString(), 15)} tokens
    │   Actual:       ${this.padRight(this.metrics.totalActual.toLocaleString(), 15)} tokens
    │   Saved:        ${this.padRight(this.metrics.tokensSaved.toLocaleString(), 15)} (${this.metrics.tokensSavedPercent.toFixed(1)}%)
    │                                                       │
    │ COST:                                               │
    │   Estimated:    $${this.padRight(this.metrics.estimatedCost.toFixed(4), 13)} │
    │   Actual:       $${this.padRight(this.metrics.actualCost.toFixed(4), 13)} │
    │   ${this.metrics.costDelta > 0 ? '✅ Saved' : '❌ Over'}:       $${this.padRight(Math.abs(this.metrics.costDelta).toFixed(4), 13)} (${this.metrics.costDeltaPercent.toFixed(1)}%)
    │                                                       │
    │ PERFORMANCE:                                        │
    │   Avg tokens/call: ${this.padRight(this.metrics.averageTokensPerCall.toFixed(0), 8)}                   │
    │   Avg time/call:   ${this.padRight((this.metrics.averageTimePerCall).toFixed(0), 8)} ms
    │                                                       │
    └─────────────────────────────────────────────────────┘
    `);
  }
  
  /**
   * Print comprehensive final report
   */
  public printFinalReport(): void {
    const elapsedMs = new Date().getTime() - this.startTime.getTime();
    const elapsedSeconds = (elapsedMs / 1000).toFixed(1);
    
    console.log(`
    ════════════════════════════════════════════════════════════════
                    ✅ FINAL TOKEN REPORT
    ════════════════════════════════════════════════════════════════
    
    TOKENS:
      Estimated total:  ${this.metrics.totalEstimated.toLocaleString().padEnd(20)} tokens
      Actual total:     ${this.metrics.totalActual.toLocaleString().padEnd(20)} tokens
      ─────────────────────────────────────────
      Saved:            ${this.metrics.tokensSaved.toLocaleString().padEnd(20)} (${this.metrics.tokensSavedPercent.toFixed(1)}%)
      Estimation error: ${(Math.abs(100 - (this.metrics.estimationAccuracy * 100))).toFixed(1)}%
    
    COST ANALYSIS:
      Estimated total:  $${this.metrics.estimatedCost.toFixed(4).padEnd(22)}
      Actual total:     $${this.metrics.actualCost.toFixed(4).padEnd(22)}
      ─────────────────────────────────────────
      ${this.metrics.costDelta > 0 ? '✅ SAVINGS' : '❌ OVERAGE'}: $${Math.abs(this.metrics.costDelta).toFixed(4).padEnd(22)} (${Math.abs(this.metrics.costDeltaPercent).toFixed(1)}%)
    
    API CALLS:
      Total calls:      ${this.metrics.calls}
      Avg tokens/call:  ${this.metrics.averageTokensPerCall.toFixed(0)}
      Avg cost/call:    $${this.metrics.averageCostPerCall.toFixed(6)}
      Avg time/call:    ${this.metrics.averageTimePerCall.toFixed(0)}ms
    
    EXECUTION:
      Total elapsed:    ${elapsedSeconds}s
      Model used:       ${this.model}
    
    ════════════════════════════════════════════════════════════════
    `);
  }
  
  /**
   * Get current metrics
   */
  public getMetrics(): TokenMetrics {
    return { ...this.metrics };
  }
  
  /**
   * Export call log for auditing
   */
  public exportCallLog(): TokenCall[] {
    return [...this.callLog];
  }
  
  /**
   * Get savings summary for inline display
   */
  public getSummarySentence(): string {
    const saved = this.metrics.costDelta > 0 ? '✅' : '❌';
    return `${saved} Used ${this.metrics.totalActual.toLocaleString()} tokens ` +
           `(estimated ${this.metrics.totalEstimated.toLocaleString()}, ` +
           `${this.metrics.costDelta > 0 ? 'saved' : 'over'} $${Math.abs(this.metrics.costDelta).toFixed(4)})`;
  }
  
  /**
   * Compare against baseline (e.g., Claude Sonnet without optimization)
   */
  public compareToBaseline(baselineCostPerMillion: number): {
    savings: number;
    savingsPercent: number;
  } {
    const baselineCost = this.metrics.totalActual * 
                         (baselineCostPerMillion / 1_000_000);
    
    return {
      savings: baselineCost - this.metrics.actualCost,
      savingsPercent: ((baselineCost - this.metrics.actualCost) / baselineCost) * 100
    };
  }
  
  private padRight(str: string, width: number): string {
    return str + ' '.repeat(Math.max(0, width - str.length));
  }
}
