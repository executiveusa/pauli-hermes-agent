const fs = require('fs');
const path = require('path');

const ROUTINE_DIR = path.join(__dirname, 'routines');

function out(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

function fail(message, code = 1) {
  out({ ok: false, error: message });
  process.exit(code);
}

function safeRoutineName(name) {
  if (!name || !/^[a-zA-Z0-9._-]+$/.test(name)) {
    throw new Error('routine_name must contain only letters, numbers, dot, underscore, or dash');
  }
  return path.join(ROUTINE_DIR, `${name}.json`);
}

function readRequest() {
  const raw = fs.readFileSync(0, 'utf8').trim();
  if (!raw) throw new Error('JSON request is required on stdin');
  return JSON.parse(raw);
}

function makeConfig(HyperAgent) {
  const provider = process.env.HYPERAGENT_LLM_PROVIDER || 'openai';
  const model = process.env.HYPERAGENT_LLM_MODEL || 'gpt-4o';
  const browserProvider = process.env.HYPERAGENT_BROWSER_PROVIDER === 'Hyperbrowser'
    ? 'Hyperbrowser'
    : undefined;
  const config = { llm: { provider, model } };
  if (browserProvider) config.browserProvider = browserProvider;
  return new HyperAgent(config);
}

async function main() {
  let pkg;
  try {
    pkg = require('@hyperbrowser/agent');
  } catch (err) {
    fail(`HyperAgent runtime is not installed. Run npm install in integrations/hyperagent first. ${err.message}`, 2);
  }

  const { HyperAgent } = pkg;
  if (process.argv.includes('--self-test')) {
    const proto = HyperAgent && HyperAgent.prototype ? HyperAgent.prototype : {};
    out({
      ok: Boolean(HyperAgent),
      package: '@hyperbrowser/agent',
      expected_version: '1.1.2',
      capabilities: {
        constructor: typeof HyperAgent === 'function',
        executeTask: typeof proto.executeTask === 'function',
        newPage: typeof proto.newPage === 'function',
        closeAgent: typeof proto.closeAgent === 'function'
      }
    });
    return;
  }

  const request = readRequest();
  const allowed = new Set(['task', 'perform', 'ai', 'extract', 'replay']);
  if (!allowed.has(request.action)) throw new Error(`Unsupported action: ${request.action}`);

  fs.mkdirSync(ROUTINE_DIR, { recursive: true });
  const agent = makeConfig(HyperAgent);

  try {
    if (request.action === 'task') {
      if (!request.task) throw new Error('task is required');
      const result = await agent.executeTask(request.task);
      out({ ok: true, action: 'task', output: result.output ?? result });
      return;
    }

    const page = await agent.newPage();

    if (request.action === 'replay') {
      const cachePath = safeRoutineName(request.routine_name);
      if (!fs.existsSync(cachePath)) throw new Error(`Routine not found: ${request.routine_name}`);
      const cache = JSON.parse(fs.readFileSync(cachePath, 'utf8'));
      const result = await page.runFromActionCache(cache, {
        maxXPathRetries: Number.isInteger(request.max_xpath_retries) ? request.max_xpath_retries : 3,
        debug: Boolean(request.debug)
      });
      out({ ok: true, action: 'replay', routine_name: request.routine_name, result });
      return;
    }

    if (!request.url) throw new Error('url is required');
    await page.goto(request.url, { waitUntil: request.wait_until || 'load' });

    if (request.action === 'perform') {
      if (!request.instruction) throw new Error('instruction is required');
      const result = await page.perform(request.instruction);
      out({ ok: true, action: 'perform', result });
      return;
    }

    if (request.action === 'extract') {
      if (!request.instruction) throw new Error('instruction is required');
      const { z } = require('zod');
      const result = await page.extract(request.instruction, z.any());
      out({ ok: true, action: 'extract', result });
      return;
    }

    if (request.action === 'ai') {
      if (!request.instruction) throw new Error('instruction is required');
      const result = await page.ai(request.instruction, {
        useDomCache: request.use_dom_cache !== false,
        enableVisualMode: Boolean(request.enable_visual_mode)
      });
      let savedRoutine = null;
      if (request.routine_name && result.actionCache) {
        const cachePath = safeRoutineName(request.routine_name);
        fs.writeFileSync(cachePath, `${JSON.stringify(result.actionCache, null, 2)}\n`, { mode: 0o600 });
        savedRoutine = path.basename(cachePath);
      }
      out({
        ok: true,
        action: 'ai',
        output: result.output ?? result,
        routine_saved: savedRoutine
      });
      return;
    }
  } finally {
    if (agent && typeof agent.closeAgent === 'function') {
      await agent.closeAgent();
    }
  }
}

main().catch((err) => fail(err && err.message ? err.message : String(err)));
