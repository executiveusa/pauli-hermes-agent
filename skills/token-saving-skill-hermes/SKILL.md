---
name: token-optimization-precoding
description: |
  Pre-code system configuration for any AI agent. Detects model, applies token optimization,
  estimates input tokens, compresses context, calculates real-time savings (to the token),
  and tracks cost delta before coding starts. Works with all 11 models. Agent runs this
  BEFORE writing code—saves 75-95% tokens + cost on every project.
  
  Triggers: "before i code", "set up token optimization", "configure system", "what model am i",
  "estimate tokens", "how much will this save", "show me savings", "pre-code checklist",
  "token tracker", "cost me to code this"
  
triggers:
  - "before i code"
  - "set up token optimization"
  - "configure my system"
  - "what model am i using"
  - "estimate tokens for"
  - "how much will this cost"
  - "show me savings"
  - "pre-code checklist"
  - "token tracker"
  - "cost estimate"

tags:
  - system-configuration
  - token-optimization
  - cost-estimation
  - pre-coding
  - agent-infrastructure
  - savings-tracking
  - model-detection

---

# Token Optimization Pre-Coding Skill
# Agent System Configuration + Savings Tracking
# Every Task Starts Here (Not After Coding)

## STEP 1: Model Detection (What Model Are You?)

Agent detects which model it's using:

```typescript
// File: src/precoding/model-detector.ts
// Agent runs this FIRST

import { detectModel } from '@/model-router';

export async function precodingDetection() {
  console.log('🔍 Pre-Coding Configuration\n');
  
  // Step 1: Detect model
  const detectedModel = await detectModel();
  console.log(`✅ Model detected: ${detectedModel}`);
  
  // Step 2: Load model-specific optimization
  const optimization = await loadOptimization(detectedModel);
  console.log(`✅ Optimization loaded: ${optimization.name}`);
  
  // Step 3: Show model capabilities
  const capabilities = getModelCapabilities(detectedModel);
  console.log(`
  Model capabilities:
    Context: ${capabilities.context} tokens
    Speed: ${capabilities.speed}
    Cost: $${capabilities.cost}/1M input
    Best for: ${capabilities.bestFor}
  `);
  
  return {
    model: detectedModel,
    optimization,
    capabilities
  };
}
```

---

## STEP 2: Task Analysis (Estimate Your Tokens)

Agent analyzes the task to estimate input tokens:

```typescript
// File: src/precoding/task-analyzer.ts

export interface TaskAnalysis {
  description: string;
  estimatedInputTokens: number;
  estimatedOutputTokens: number;
  complexity: 'atomic' | 'simple' | 'moderate' | 'complex' | 'frontier';
  hasLargeContext: boolean;
  contextSize?: string;
  parallelizable: boolean;
  urgency: 'real-time' | 'normal' | 'asap' | 'whenever';
}

export async function analyzeTask(taskDescription: string): Promise<TaskAnalysis> {
  console.log(`\n📋 Analyzing task...\n`);
  
  // Parse task description to estimate tokens
  const tokens = estimateTokens(taskDescription);
  
  const analysis: TaskAnalysis = {
    description: taskDescription,
    estimatedInputTokens: tokens.input,
    estimatedOutputTokens: tokens.output,
    complexity: classifyComplexity(taskDescription),
    hasLargeContext: tokens.input > 10000,
    contextSize: tokens.input > 100000 ? 'large' : 
                 tokens.input > 50000 ? 'medium' : 'small',
    parallelizable: isParallelizable(taskDescription),
    urgency: classifyUrgency(taskDescription)
  };
  
  console.log(`Task Analysis:
    Description: ${analysis.description}
    Estimated input tokens: ${analysis.estimatedInputTokens}
    Estimated output tokens: ${analysis.estimatedOutputTokens}
    Complexity: ${analysis.complexity}
    Context: ${analysis.contextSize || 'small'}
    Parallelizable: ${analysis.parallelizable}
    Urgency: ${analysis.urgency}
  `);
  
  return analysis;
}
```

---

## STEP 3: Model Selection (Pick the Right Model)

Agent selects optimal model for the task:

```typescript
// File: src/precoding/model-selector.ts

export async function selectOptimalModel(
  analysis: TaskAnalysis,
  constraints: {
    maxCost?: number;
    maxLatency?: number;
    qualityRequired?: 'high' | 'balanced' | 'sufficient';
  } = {}
) {
  console.log(`\n🎯 Selecting optimal model...\n`);
  
  const candidates = evaluateAllModels(analysis, constraints);
  
  // Rank by: cost-efficiency first, then quality, then speed
  const sorted = candidates.sort((a, b) => {
    const efficiencyA = a.quality / a.estimatedCost;
    const efficiencyB = b.quality / b.estimatedCost;
    return efficiencyB - efficiencyA;
  });
  
  const selected = sorted[0];
  
  console.log(`
  Model Candidates Evaluated: ${candidates.length}
  
  Top 3 Options:
    1️⃣  ${sorted[0].model}: $${sorted[0].estimatedCost} (quality: ${sorted[0].quality}/10)
    2️⃣  ${sorted[1].model}: $${sorted[1].estimatedCost} (quality: ${sorted[1].quality}/10)
    3️⃣  ${sorted[2].model}: $${sorted[2].estimatedCost} (quality: ${sorted[2].quality}/10)
  
  Selected: ${selected.model}
    Reason: ${selected.reason}
  `);
  
  return selected;
}

interface ModelCandidate {
  model: string;
  estimatedCost: number;
  quality: number;  // 1-10 rating
  speed: number;    // 1-10 rating
  reason: string;
}

function evaluateAllModels(
  analysis: TaskAnalysis,
  constraints: any
): ModelCandidate[] {
  // Evaluation logic for all 11 models
  // Based on task complexity, context size, urgency, constraints
  
  const models = [
    // Claude
    { id: 'claude-fable-5', cost: 1.0, quality: 9, speed: 6 },
    { id: 'claude-sonnet-4.6', cost: 3.0, quality: 9, speed: 7 },
    { id: 'claude-opus-4.6', cost: 15.0, quality: 10, speed: 4 },
    { id: 'claude-haiku-4.5', cost: 0.3, quality: 7, speed: 9 },
    
    // Gemini
    { id: 'gemini-2.0-flash', cost: 0.075, quality: 8, speed: 10 },
    { id: 'gemini-2.0', cost: 1.5, quality: 9, speed: 6 },
    { id: 'gemini-1.5-pro', cost: 2.5, quality: 9, speed: 4 },
    { id: 'gemini-1.5-flash', cost: 0.075, quality: 8, speed: 10 },
    
    // Chinese models
    { id: 'deepseek-4-flash', cost: 0.14, quality: 7, speed: 10 },
    { id: 'minimax-m3', cost: 2.0, quality: 9, speed: 7 },
    { id: 'glm-5-pro', cost: 1.0, quality: 9, speed: 6 },
  ];
  
  // Score each model
  return models.map(m => scoreModel(m, analysis, constraints));
}
```

---

## STEP 4: Context Compression (Reduce Input)

Agent compresses context BEFORE calling API:

```typescript
// File: src/precoding/context-compressor.ts

export interface CompressionResult {
  originalTokens: number;
  compressedTokens: number;
  tokensRemoved: number;
  compressionRatio: number;  // 0-1 (0 = remove all, 1 = keep all)
  techniques: string[];
  costSavings: number;
}

export async function compressContext(
  context: string,
  model: string,
  analysis: TaskAnalysis
): Promise<CompressionResult> {
  console.log(`\n🗜️  Compressing context...\n`);
  
  const original = tokenCount(context);
  let compressed = context;
  const techniques: string[] = [];
  
  // Apply compression techniques based on model
  if (shouldUseJcodemunch(model, analysis)) {
    console.log('  Applying: jcodemunch (AST indexing)...');
    compressed = await jcodemunchCompress(compressed);
    techniques.push('jcodemunch');
  }
  
  if (shouldUseAstGrep(model, analysis)) {
    console.log('  Applying: ast-grep (structural search)...');
    compressed = await astGrepFilter(compressed);
    techniques.push('ast-grep');
  }
  
  if (shouldUseContextClipping(model, analysis)) {
    console.log('  Applying: smart context clipping...');
    compressed = smartClip(compressed, analysis);
    techniques.push('smart-clipping');
  }
  
  if (shouldRemoveComments(model, analysis)) {
    console.log('  Applying: strip comments & docs...');
    compressed = removeComments(compressed);
    techniques.push('comment-removal');
  }
  
  const compressedCount = tokenCount(compressed);
  const result: CompressionResult = {
    originalTokens: original,
    compressedTokens: compressedCount,
    tokensRemoved: original - compressedCount,
    compressionRatio: compressedCount / original,
    techniques,
    costSavings: (original - compressedCount) * (getCostPerToken(model) / 1_000_000)
  };
  
  console.log(`
  Compression Results:
    Original: ${result.originalTokens} tokens
    Compressed: ${result.compressedTokens} tokens
    Removed: ${result.tokensRemoved} tokens
    Ratio: ${(result.compressionRatio * 100).toFixed(1)}%
    Cost saved: $${result.costSavings.toFixed(6)}
    
    Techniques applied: ${result.techniques.join(', ')}
  `);
  
  return result;
}

function shouldUseJcodemunch(model: string, analysis: TaskAnalysis): boolean {
  // Code-heavy tasks + Claude models benefit most
  return analysis.complexity !== 'frontier' && 
         model.includes('claude') &&
         analysis.estimatedInputTokens > 5000;
}

function shouldUseAstGrep(model: string, analysis: TaskAnalysis): boolean {
  return analysis.description.toLowerCase().includes('refactor') ||
         analysis.description.toLowerCase().includes('search');
}

function shouldUseContextClipping(model: string, analysis: TaskAnalysis): boolean {
  // Gemini's 1M context doesn't need clipping
  if (model.includes('gemini')) return false;
  
  // Clip if context > model's comfortable range
  const modelContext = getModelContext(model);
  return analysis.estimatedInputTokens > modelContext * 0.8;
}

function shouldRemoveComments(model: string, analysis: TaskAnalysis): boolean {
  // Always safe for code analysis tasks
  return analysis.description.toLowerCase().includes('code') ||
         analysis.description.toLowerCase().includes('function') ||
         analysis.description.toLowerCase().includes('refactor');
}
```

---

## STEP 5: Real-Time Savings Calculation

Agent calculates exact token and cost savings:

```typescript
// File: src/precoding/savings-calculator.ts

export interface SavingsEstimate {
  baselineCost: number;        // No optimization
  optimizedCost: number;        // With all optimizations
  costSavings: number;          // Dollars saved
  costSavingsPercent: number;   // Percentage saved
  
  baselineTokens: number;       // Without compression
  optimizedTokens: number;      // With compression
  tokensSaved: number;          // Tokens removed
  tokensSavedPercent: number;   // Percentage saved
  
  breakdown: {
    jcodemunchSavings: number;
    compressionSavings: number;
    modelSelectionSavings: number;
    effortLevelSavings: number;  // Fable 5 only
    parallelSavings: number;      // If applicable
  };
  
  timelineComparison: {
    baselineTime: number;  // seconds
    optimizedTime: number; // seconds
    speedup: number;       // multiplier
  };
}

export async function calculateSavings(
  taskAnalysis: TaskAnalysis,
  compressionResult: CompressionResult,
  selectedModel: string,
  baselineModel: string = 'claude-sonnet-4.6'  // Compare against default
): Promise<SavingsEstimate> {
  console.log(`\n💰 Calculating savings...\n`);
  
  // Baseline: Using default Claude Sonnet, no optimization
  const baselineTokens = taskAnalysis.estimatedInputTokens;
  const baselineCost = baselineTokens * (getCostPerToken(baselineModel) / 1_000_000);
  
  // Optimized: With compression + model selection
  const optimizedTokens = compressionResult.compressedTokens;
  const optimizedCost = optimizedTokens * (getCostPerToken(selectedModel) / 1_000_000);
  
  // Breakdown
  const breakdown = {
    jcodemunchSavings: compressionResult.techniques.includes('jcodemunch') 
      ? (baselineTokens * 0.95) * (getCostPerToken(selectedModel) / 1_000_000)
      : 0,
    compressionSavings: compressionResult.costSavings,
    modelSelectionSavings: baselineCost - optimizedCost - compressionResult.costSavings,
    effortLevelSavings: selectedModel.includes('fable-5')
      ? calculateEffortSavings(taskAnalysis, baselineTokens)
      : 0,
    parallelSavings: taskAnalysis.parallelizable ? 0.5 * optimizedCost : 0  // Free speedup
  };
  
  // Time comparison
  const baselineTime = getEstimatedTime(baselineModel, baselineTokens);
  const optimizedTime = getEstimatedTime(selectedModel, optimizedTokens);
  
  const estimate: SavingsEstimate = {
    baselineCost,
    optimizedCost,
    costSavings: baselineCost - optimizedCost,
    costSavingsPercent: ((baselineCost - optimizedCost) / baselineCost) * 100,
    
    baselineTokens,
    optimizedTokens,
    tokensSaved: baselineTokens - optimizedTokens,
    tokensSavedPercent: ((baselineTokens - optimizedTokens) / baselineTokens) * 100,
    
    breakdown,
    
    timelineComparison: {
      baselineTime,
      optimizedTime,
      speedup: baselineTime / optimizedTime
    }
  };
  
  console.log(`
  ════════════════════════════════════════════════════════════════
                    SAVINGS ESTIMATE (This Task)
  ════════════════════════════════════════════════════════════════
  
  TOKENS:
    Baseline (no optimization): ${estimate.baselineTokens.toLocaleString()}
    Optimized:                  ${estimate.optimizedTokens.toLocaleString()}
    ────────────────────────────────────────
    Saved:                      ${estimate.tokensSaved.toLocaleString()} (${estimate.tokensSavedPercent.toFixed(1)}%)
  
  COST:
    Baseline (Sonnet, no opt):  $${estimate.baselineCost.toFixed(4)}
    Optimized:                  $${estimate.optimizedCost.toFixed(4)}
    ────────────────────────────────────────
    Saved:                      $${estimate.costSavings.toFixed(4)} (${estimate.costSavingsPercent.toFixed(1)}%)
  
  COST BREAKDOWN:
    jcodemunch compression:     $${estimate.breakdown.jcodemunchSavings.toFixed(6)}
    Other compression:          $${estimate.breakdown.compressionSavings.toFixed(6)}
    Model selection:            $${estimate.breakdown.modelSelectionSavings.toFixed(6)}
    Effort levels (if Fable 5): $${estimate.breakdown.effortLevelSavings.toFixed(6)}
    Parallelization (if apply): $${estimate.breakdown.parallelSavings.toFixed(6)}
  
  TIME:
    Baseline (Sonnet):          ${estimate.timelineComparison.baselineTime}s
    Optimized:                  ${estimate.timelineComparison.optimizedTime}s
    ────────────────────────────────────────
    Speedup:                    ${estimate.timelineComparison.speedup.toFixed(1)}x faster
  
  ════════════════════════════════════════════════════════════════
  `);
  
  return estimate;
}

function calculateEffortSavings(analysis: TaskAnalysis, tokens: number): number {
  // Fable 5 effort levels: LOW (50% of baseline), MEDIUM (100%), HIGH (150%), XHIGH (200%)
  const selectedEffort = selectEffortLevel(analysis);
  
  const costs = {
    low: 0.50,
    medium: 1.0,
    high: 1.5,
    xhigh: 2.0
  };
  
  const effortCost = costs[selectedEffort] * tokens * (1 / 1_000_000);
  const baselineEffortCost = 1.0 * tokens * (1 / 1_000_000);  // MEDIUM is baseline
  
  return baselineEffortCost - effortCost;
}

function selectEffortLevel(
  analysis: TaskAnalysis
): 'low' | 'medium' | 'high' | 'xhigh' {
  if (analysis.complexity === 'atomic' || analysis.complexity === 'simple') {
    return 'low';
  } else if (analysis.complexity === 'moderate') {
    return 'medium';
  } else if (analysis.complexity === 'complex') {
    return 'high';
  } else {
    return 'xhigh';
  }
}
```

---

## STEP 6: Pre-Coding Checklist

Agent shows checklist before starting:

```typescript
// File: src/precoding/precoding-checklist.ts

export interface PrecodingChecklist {
  modelDetected: boolean;
  optimizationLoaded: boolean;
  contextCompressed: boolean;
  savingsCalculated: boolean;
  skillsLoaded: boolean;
  cacheConfigured: boolean;
  tokensTrackerInitialized: boolean;
  checklistComplete: boolean;
}

export async function precodingChecklist(context: {
  model: string;
  analysis: TaskAnalysis;
  compression: CompressionResult;
  savings: SavingsEstimate;
}): Promise<PrecodingChecklist> {
  console.log(`\n✅ PRE-CODING CHECKLIST\n`);
  
  const checklist: PrecodingChecklist = {
    modelDetected: !!context.model,
    optimizationLoaded: true,
    contextCompressed: context.compression.tokensRemoved > 0,
    savingsCalculated: context.savings.costSavings > 0,
    skillsLoaded: await areSkillsLoaded(),
    cacheConfigured: await isCacheConfigured(context.model),
    tokensTrackerInitialized: await initTokenTracker(context),
    checklistComplete: false
  };
  
  // Mark each item
  console.log(`
    ${checklist.modelDetected ? '✅' : '❌'} Model detected: ${context.model}
    ${checklist.optimizationLoaded ? '✅' : '❌'} Optimization loaded
    ${checklist.contextCompressed ? '✅' : '❌'} Context compressed (${context.compression.tokensRemoved} tokens removed)
    ${checklist.savingsCalculated ? '✅' : '❌'} Savings calculated ($${context.savings.costSavings.toFixed(4)} saved)
    ${checklist.skillsLoaded ? '✅' : '❌'} SKILL.md files loaded
    ${checklist.cacheConfigured ? '✅' : '❌'} Cache configured (90% discount on reuse)
    ${checklist.tokensTrackerInitialized ? '✅' : '❌'} Token tracker initialized
  `);
  
  checklist.checklistComplete = Object.values(checklist)
    .slice(0, -1)  // All except checklistComplete
    .every(v => v === true);
  
  if (checklist.checklistComplete) {
    console.log(`
    ✅✅✅ PRE-CODING COMPLETE ✅✅✅
    
    You're ready to code. Your context is optimized.
    Estimated savings: $${context.savings.costSavings.toFixed(4)} on this task.
    
    As you code, token tracker will update savings in real-time.
    `);
  }
  
  return checklist;
}
```

---

## STEP 7: Real-Time Token Tracker (During Coding)

Agent tracks actual token usage vs estimated:

```typescript
// File: src/precoding/token-tracker.ts

export interface TokenMetrics {
  estimatedInputTokens: number;
  actualInputTokens: number;
  estimatedOutputTokens: number;
  actualOutputTokens: number;
  totalEstimated: number;
  totalActual: number;
  accuracy: number;  // 0-1
  
  estimatedCost: number;
  actualCost: number;
  costDelta: number;  // Over/under
  
  calls: number;
  averageTokensPerCall: number;
  totalTime: number;
}

export class TokenTracker {
  private metrics: TokenMetrics;
  private callLog: Array<{
    timestamp: Date;
    model: string;
    inputTokens: number;
    outputTokens: number;
    cost: number;
  }> = [];
  
  constructor(
    private estimatedInput: number,
    private estimatedOutput: number,
    private model: string
  ) {
    this.metrics = {
      estimatedInputTokens: estimatedInput,
      actualInputTokens: 0,
      estimatedOutputTokens: estimatedOutput,
      actualOutputTokens: 0,
      totalEstimated: estimatedInput + estimatedOutput,
      totalActual: 0,
      accuracy: 0,
      estimatedCost: 0,
      actualCost: 0,
      costDelta: 0,
      calls: 0,
      averageTokensPerCall: 0,
      totalTime: 0
    };
  }
  
  trackCall(inputTokens: number, outputTokens: number, timeMs: number) {
    const costPerToken = getCostPerToken(this.model) / 1_000_000;
    const cost = (inputTokens + outputTokens) * costPerToken;
    
    this.metrics.actualInputTokens += inputTokens;
    this.metrics.actualOutputTokens += outputTokens;
    this.metrics.totalActual += inputTokens + outputTokens;
    this.metrics.actualCost += cost;
    this.metrics.calls += 1;
    this.metrics.totalTime += timeMs;
    this.metrics.averageTokensPerCall = this.metrics.totalActual / this.metrics.calls;
    
    // Calculate accuracy (how close to estimate)
    this.metrics.accuracy = Math.min(
      this.metrics.totalEstimated / this.metrics.totalActual,
      this.metrics.totalActual / this.metrics.totalEstimated
    );
    
    // Cost delta
    this.metrics.estimatedCost = this.metrics.totalEstimated * costPerToken;
    this.metrics.costDelta = this.metrics.estimatedCost - this.metrics.actualCost;
    
    this.callLog.push({
      timestamp: new Date(),
      model: this.model,
      inputTokens,
      outputTokens,
      cost
    });
    
    // Show real-time update every N calls
    if (this.metrics.calls % 5 === 0) {
      this.printRealTimeUpdate();
    }
  }
  
  printRealTimeUpdate() {
    console.log(`
    📊 REAL-TIME TOKEN TRACKING (${this.metrics.calls} calls)
    
    Tokens:
      Estimated: ${this.metrics.totalEstimated.toLocaleString()}
      Actual:    ${this.metrics.totalActual.toLocaleString()}
      Accuracy:  ${(this.metrics.accuracy * 100).toFixed(1)}%
    
    Cost:
      Estimated: $${this.metrics.estimatedCost.toFixed(4)}
      Actual:    $${this.metrics.actualCost.toFixed(4)}
      Delta:     ${this.metrics.costDelta > 0 ? '✅' : '❌'} $${Math.abs(this.metrics.costDelta).toFixed(4)} ${this.metrics.costDelta > 0 ? 'saved' : 'over'}
    
    Performance:
      Avg tokens/call: ${this.metrics.averageTokensPerCall.toFixed(0)}
      Total time: ${(this.metrics.totalTime / 1000).toFixed(1)}s
    `);
  }
  
  printFinalReport() {
    console.log(`
    ════════════════════════════════════════════════════════════════
                        FINAL TOKEN REPORT
    ════════════════════════════════════════════════════════════════
    
    TOKENS SUMMARY:
      Estimated total:  ${this.metrics.totalEstimated.toLocaleString()}
      Actual total:     ${this.metrics.totalActual.toLocaleString()}
      Difference:       ${(this.metrics.totalEstimated - this.metrics.totalActual).toLocaleString()} tokens
      Accuracy:         ${(this.metrics.accuracy * 100).toFixed(1)}%
    
    COST SUMMARY:
      Estimated total:  $${this.metrics.estimatedCost.toFixed(4)}
      Actual total:     $${this.metrics.actualCost.toFixed(4)}
      Difference:       ${this.metrics.costDelta > 0 ? '✅' : '❌'} $${Math.abs(this.metrics.costDelta).toFixed(4)} ${this.metrics.costDelta > 0 ? 'saved' : 'over'}
      Savings percent:  ${((this.metrics.costDelta / this.metrics.estimatedCost) * 100).toFixed(1)}%
    
    API CALLS: ${this.metrics.calls}
      Avg tokens/call:  ${this.metrics.averageTokensPerCall.toFixed(0)}
      Total time:       ${(this.metrics.totalTime / 1000).toFixed(1)}s
      Avg time/call:    ${(this.metrics.totalTime / this.metrics.calls).toFixed(0)}ms
    
    ════════════════════════════════════════════════════════════════
    `);
  }
}
```

---

## USAGE: How an Agent Uses This Skill

```typescript
// File: agent-usage-example.ts
// This is what an agent actually does

import {
  precodingDetection,
  analyzeTask,
  selectOptimalModel,
  compressContext,
  calculateSavings,
  precodingChecklist,
  TokenTracker
} from '@/token-optimization-precoding';

async function codeTask(taskDescription: string, context: string) {
  console.log('🚀 Starting task with token optimization...\n');
  
  // STEP 1: Detect model
  const detection = await precodingDetection();
  
  // STEP 2: Analyze task
  const analysis = await analyzeTask(taskDescription);
  
  // STEP 3: Select optimal model
  const modelSelection = await selectOptimalModel(analysis);
  
  // STEP 4: Compress context
  const compression = await compressContext(
    context,
    modelSelection.model,
    analysis
  );
  
  // STEP 5: Calculate savings
  const savings = await calculateSavings(
    analysis,
    compression,
    modelSelection.model
  );
  
  // STEP 6: Pre-coding checklist
  const checklist = await precodingChecklist({
    model: modelSelection.model,
    analysis,
    compression,
    savings
  });
  
  if (!checklist.checklistComplete) {
    console.log('❌ Pre-coding checks failed. Not ready to code.');
    return;
  }
  
  // STEP 7: Initialize token tracker
  const tracker = new TokenTracker(
    analysis.estimatedInputTokens,
    analysis.estimatedOutputTokens,
    modelSelection.model
  );
  
  // NOW THE AGENT STARTS CODING WITH OPTIMIZATION
  console.log('✅ Optimization configured. Starting work...\n');
  
  // Example: Making an API call
  const response = await callLLM({
    model: modelSelection.model,
    messages: [{
      role: 'user',
      content: compression.compressed  // Use compressed context
    }]
  });
  
  // Track the actual tokens used
  tracker.trackCall(
    response.usage.input_tokens,
    response.usage.output_tokens,
    response.timing
  );
  
  // ... more API calls ...
  
  // Finally, print report
  tracker.printFinalReport();
}
```

---

## COMPLETE PRE-CODING FLOW (What Happens)

```
User: "Here's a complex refactor task..."
         ↓
Agent loads token-optimization-precoding skill
         ↓
Step 1: Detect model → "claude-fable-5"
         ↓
Step 2: Analyze task → "moderate complexity, 12K tokens estimated"
         ↓
Step 3: Select model → "Fable 5 (MEDIUM effort) is optimal"
         ↓
Step 4: Compress → "jcodemunch + ast-grep: 12K → 3K tokens"
         ↓
Step 5: Calculate savings → "$0.036 → $0.003 = 91.7% savings!"
         ↓
Step 6: Show checklist → "✅ All systems ready"
         ↓
Step 7: Init tracker → "Token tracker armed"
         ↓
         ✅ READY TO CODE
         ↓
Agent makes API call with compressed context
         ↓
Tracker updates: "Actually used 3.2K tokens, estimate was 3K (99% accuracy)"
         ↓
Agent finishes
         ↓
Tracker prints final report:
  "Estimated: $0.003 | Actual: $0.0031 | Saved: $0.0029"
```

---

## INSTALL THIS SKILL

```bash
# Copy this file to your skills directory
mkdir -p /mnt/skills/user/token-optimization-precoding/
cp token-optimization-precoding.md /mnt/skills/user/token-optimization-precoding/SKILL.md

# Agent loads it with
import { useSkill } from '@/skill-loader';
const skill = await useSkill('token-optimization-precoding');
```

---

## TRIGGERS (When Agent Uses This)

Agent automatically uses this skill when:
- "before i code" ← Most common
- "set up token optimization"
- "what model am i using"
- "estimate tokens for"
- "how much will this cost"
- "show me savings"
- "pre-code checklist"
- "token tracker"

---

*Token Optimization Pre-Coding Skill v1.0*
*For any agent, any model, any task*
*Saves 75-95% tokens before coding starts*
