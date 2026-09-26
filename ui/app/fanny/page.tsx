'use client';

import { useMemo, useState } from 'react';
import styles from './fanny.module.css';

type Mention = { source: string; text: string; category: string; sentiment: 'Positive' | 'Neutral' | 'Negative'; confidence: number; needsReview: boolean };

const mentions: Mention[] = [
  { source: 'X', text: 'Banorte app transfer worked quickly this morning.', category: 'Digital experience', sentiment: 'Positive', confidence: 94, needsReview: false },
  { source: 'News', text: 'Analysts discuss Banorte quarterly performance and credit growth.', category: 'Financial performance', sentiment: 'Neutral', confidence: 91, needsReview: false },
  { source: 'Facebook', text: 'The branch line was long and I could not resolve my card issue.', category: 'Customer service', sentiment: 'Negative', confidence: 97, needsReview: false },
  { source: 'Forum', text: 'Is this Banorte promotion legitimate?', category: 'Fraud / trust', sentiment: 'Neutral', confidence: 73, needsReview: true },
  { source: 'Instagram', text: 'Thank you to the local Banorte team for supporting the community event.', category: 'Community impact', sentiment: 'Positive', confidence: 96, needsReview: false },
  { source: 'Blog', text: 'A comparison of Mexican banking apps includes Banorte among five options.', category: 'Competitive mention', sentiment: 'Neutral', confidence: 84, needsReview: true },
];

const stages = ['Collect', 'Filter', 'Classify', 'Verify', 'Summarize', 'Review'];

export default function FannyPage() {
  const [step, setStep] = useState(-1);
  const [running, setRunning] = useState(false);
  const [approved, setApproved] = useState(false);
  const [fileName, setFileName] = useState('Built-in sample mentions');
  const [notes, setNotes] = useState('');

  const totals = useMemo(() => ({
    total: mentions.length,
    positive: mentions.filter((m) => m.sentiment === 'Positive').length,
    neutral: mentions.filter((m) => m.sentiment === 'Neutral').length,
    negative: mentions.filter((m) => m.sentiment === 'Negative').length,
    review: mentions.filter((m) => m.needsReview).length,
  }), []);

  function speak(text: string) {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) return;
    window.speechSynthesis.cancel();
    const voice = new SpeechSynthesisUtterance(text);
    voice.rate = 0.96;
    voice.pitch = 1.03;
    window.speechSynthesis.speak(voice);
  }

  async function runDemo() {
    setApproved(false);
    setRunning(true);
    speak('I will organize these mentions, identify uncertainty, and prepare a reviewable report. I will not send or change anything without approval.');
    for (let i = 0; i < stages.length; i += 1) {
      setStep(i);
      await new Promise((resolve) => setTimeout(resolve, 700));
    }
    setRunning(false);
    speak('I reviewed six mentions. Two need human judgment. The draft is ready for review.');
  }

  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <div>
          <span className={styles.eyebrow}>PAULI DIGITAL WORKER · WORKING SANDBOX</span>
          <h1>Meet Fanny.</h1>
          <p className={styles.lede}>A media intelligence agent that turns scattered mentions into a reviewed weekly report.</p>
          <div className={styles.actions}>
            <button className={styles.primary} onClick={runDemo} disabled={running}>{running ? 'Fanny is working…' : 'Run the demo'}</button>
            <button className={styles.secondary} onClick={() => speak('Hello. I am Fanny. I organize media mentions, explain uncertainty, and ask for approval before anything is submitted.')}>Hear Fanny</button>
          </div>
        </div>
        <aside className={styles.avatarCard}>
          <div className={`${styles.avatar} ${running ? styles.active : ''}`}><span>F</span><i /></div>
          <strong>Fanny</strong>
          <p>{running ? `Working: ${stages[step]}` : approved ? 'Report approved' : 'Ready to help'}</p>
          <small>Sandbox mode · No external actions</small>
        </aside>
      </section>

      <section className={styles.workspace}>
        <article className={styles.panel}>
          <div className={styles.heading}><div><span>1 · SHOW THE WORK</span><h2>Give Fanny a sample</h2></div><em>Read-only</em></div>
          <label className={styles.upload}>
            <input type="file" accept=".csv,.txt,.json" onChange={(e) => setFileName(e.target.files?.[0]?.name || 'Built-in sample mentions')} />
            <b>Upload CSV, text, or JSON</b><small>{fileName}</small>
          </label>
          <textarea value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="Tell Fanny what matters in this report…" />
        </article>

        <article className={styles.panel}>
          <span className={styles.label}>2 · WATCH THE PROCESS</span><h2>Visible, bounded work</h2>
          <div className={styles.pipeline}>{stages.map((stage, index) => <div key={stage} className={`${styles.stage} ${index < step || (!running && step === stages.length - 1) ? styles.done : ''} ${running && index === step ? styles.current : ''}`}><small>{index + 1}</small><b>{stage}</b></div>)}</div>
          <p className={styles.guardrail}>Fanny may analyze and draft. Sending, deleting, changing source records, or accessing credentials requires separate approval.</p>
        </article>
      </section>

      <section className={styles.results}>
        <div className={styles.resultHeading}><div><span>3 · REVIEW THE RESULT</span><h2>Weekly media signal brief</h2></div><em>Prototype evidence · browser session</em></div>
        <div className={styles.stats}>{[
          ['Mentions', totals.total], ['Positive', totals.positive], ['Neutral', totals.neutral], ['Negative', totals.negative], ['Need review', totals.review]
        ].map(([label, value]) => <div key={String(label)}><strong>{value}</strong><span>{label}</span></div>)}</div>
        <div className={styles.tableWrap}><table><thead><tr><th>Source</th><th>Mention</th><th>Category</th><th>Signal</th><th>Confidence</th></tr></thead><tbody>{mentions.map((m) => <tr key={m.text}><td>{m.source}</td><td>{m.text}</td><td>{m.category}{m.needsReview && <mark>Review</mark>}</td><td><b>{m.sentiment}</b></td><td>{m.confidence}%</td></tr>)}</tbody></table></div>
        <div className={styles.summaries}>
          <article><span>WHAT FANNY FOUND</span><p>Digital experience and community signals are favorable. The strongest negative signal concerns branch support. Two ambiguous items were withheld for human judgment.</p></article>
          <article><span>WHAT FANNY DID NOT DO</span><p>No source records were changed. No report was sent. No customer credentials or external systems were accessed.</p></article>
          <article><span>RECOMMENDED NEXT STEP</span><p>Review the fraud-related question and competitive comparison, then approve the report draft for team delivery.</p></article>
        </div>
        <div className={styles.approval}>
          <div><strong>{approved ? 'Approved for this demonstration' : 'Human review required'}</strong><span>{approved ? 'An approval receipt is recorded in this browser session.' : 'Fanny stops here until a person decides.'}</span></div>
          <div className={styles.actions}><button className={styles.secondary} onClick={() => { setApproved(false); speak('The report was returned for correction.'); }}>Request correction</button><button className={styles.primary} onClick={() => { setApproved(true); speak('Approval recorded. In production, only the specifically authorized next step would be released.'); }}>Approve report</button></div>
        </div>
      </section>
    </main>
  );
}
