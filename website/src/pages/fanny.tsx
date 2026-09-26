import React, {useMemo, useState} from 'react';
import Layout from '@theme/Layout';
import styles from './fanny.module.css';

type Mention = {
  source: string;
  text: string;
  category: string;
  sentiment: 'Positive' | 'Neutral' | 'Negative';
  confidence: number;
  needsReview: boolean;
};

const sampleMentions: Mention[] = [
  {source: 'X', text: 'Banorte app transfer worked quickly this morning.', category: 'Digital experience', sentiment: 'Positive', confidence: 94, needsReview: false},
  {source: 'News', text: 'Analysts discuss Banorte quarterly performance and credit growth.', category: 'Financial performance', sentiment: 'Neutral', confidence: 91, needsReview: false},
  {source: 'Facebook', text: 'The branch line was long and I could not resolve my card issue.', category: 'Customer service', sentiment: 'Negative', confidence: 97, needsReview: false},
  {source: 'Forum', text: 'Is this Banorte promotion legitimate?', category: 'Fraud / trust', sentiment: 'Neutral', confidence: 73, needsReview: true},
  {source: 'Instagram', text: 'Thank you to the local Banorte team for supporting the community event.', category: 'Community impact', sentiment: 'Positive', confidence: 96, needsReview: false},
  {source: 'Blog', text: 'A comparison of Mexican banking apps includes Banorte among five options.', category: 'Competitive mention', sentiment: 'Neutral', confidence: 84, needsReview: true},
];

const stages = ['Collect', 'Filter', 'Classify', 'Verify', 'Summarize', 'Review'];

export default function FannyPrototype(): JSX.Element {
  const [step, setStep] = useState(0);
  const [running, setRunning] = useState(false);
  const [approved, setApproved] = useState(false);
  const [fileName, setFileName] = useState('Sample media mentions');
  const [notes, setNotes] = useState('');

  const totals = useMemo(() => ({
    total: sampleMentions.length,
    positive: sampleMentions.filter((m) => m.sentiment === 'Positive').length,
    neutral: sampleMentions.filter((m) => m.sentiment === 'Neutral').length,
    negative: sampleMentions.filter((m) => m.sentiment === 'Negative').length,
    review: sampleMentions.filter((m) => m.needsReview).length,
  }), []);

  const speak = (text: string) => {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) return;
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = 0.96;
    utterance.pitch = 1.02;
    window.speechSynthesis.speak(utterance);
  };

  const runDemo = async () => {
    setApproved(false);
    setRunning(true);
    setStep(0);
    speak('I will organize these mentions, identify uncertainty, and prepare a reviewable report. I will not send or change anything without approval.');
    for (let i = 0; i < stages.length; i += 1) {
      setStep(i);
      await new Promise((resolve) => setTimeout(resolve, 650));
    }
    setRunning(false);
    speak(`I reviewed ${totals.total} mentions. Two need human judgment. The draft is ready for review.`);
  };

  return (
    <Layout title="Agent Fanny Prototype" description="Interactive media intelligence agent demo">
      <main className={styles.page}>
        <section className={styles.hero}>
          <div>
            <span className={styles.eyebrow}>PAULI DIGITAL WORKER PROTOTYPE</span>
            <h1>Meet Fanny.</h1>
            <p className={styles.lede}>A media intelligence agent that turns scattered mentions into a reviewed weekly report.</p>
            <div className={styles.heroActions}>
              <button className={styles.primary} onClick={runDemo} disabled={running}>{running ? 'Fanny is working…' : 'Run the 60-second demo'}</button>
              <button className={styles.secondary} onClick={() => speak('Hello. I am Fanny. I organize media mentions, explain uncertainty, and ask for approval before anything is submitted.')}>Hear Fanny</button>
            </div>
          </div>
          <div className={styles.avatarCard}>
            <div className={`${styles.avatar} ${running ? styles.avatarActive : ''}`}>
              <div className={styles.face}>F</div>
              <div className={styles.pulse} />
            </div>
            <strong>Fanny</strong>
            <span>{running ? `Working: ${stages[step]}` : approved ? 'Report approved' : 'Ready to help'}</span>
            <small>Sandbox mode · No external actions</small>
          </div>
        </section>

        <section className={styles.workspace}>
          <div className={styles.panel}>
            <div className={styles.panelHeader}>
              <div><span className={styles.kicker}>1 · SHOW THE WORK</span><h2>Give Fanny a sample</h2></div>
              <span className={styles.safe}>Read-only demo</span>
            </div>
            <label className={styles.upload}>
              <input type="file" accept=".csv,.txt,.json" onChange={(event) => setFileName(event.target.files?.[0]?.name || 'Sample media mentions')} />
              <span>Upload CSV, text, or JSON</span>
              <small>{fileName}</small>
            </label>
            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} placeholder="Tell Fanny what matters in this report…" className={styles.notes} />
          </div>

          <div className={styles.panel}>
            <span className={styles.kicker}>2 · WATCH THE PROCESS</span>
            <h2>Visible, bounded work</h2>
            <div className={styles.pipeline}>
              {stages.map((stage, index) => (
                <div key={stage} className={`${styles.stage} ${index < step || (!running && step === stages.length - 1) ? styles.done : ''} ${running && index === step ? styles.current : ''}`}>
                  <span>{index + 1}</span><strong>{stage}</strong>
                </div>
              ))}
            </div>
            <p className={styles.guardrail}>Fanny can analyze and draft. Sending, deleting, changing source records, or accessing credentials requires separate approval.</p>
          </div>
        </section>

        <section className={styles.results}>
          <div className={styles.resultHeader}>
            <div><span className={styles.kicker}>3 · REVIEW THE RESULT</span><h2>Weekly media signal brief</h2></div>
            <span className={styles.timestamp}>Prototype evidence · Generated locally</span>
          </div>

          <div className={styles.stats}>
            <div><strong>{totals.total}</strong><span>Mentions</span></div>
            <div><strong>{totals.positive}</strong><span>Positive</span></div>
            <div><strong>{totals.neutral}</strong><span>Neutral</span></div>
            <div><strong>{totals.negative}</strong><span>Negative</span></div>
            <div><strong>{totals.review}</strong><span>Need review</span></div>
          </div>

          <div className={styles.tableWrap}>
            <table>
              <thead><tr><th>Source</th><th>Mention</th><th>Category</th><th>Signal</th><th>Confidence</th></tr></thead>
              <tbody>{sampleMentions.map((mention) => (
                <tr key={mention.text}>
                  <td>{mention.source}</td><td>{mention.text}</td><td>{mention.category}{mention.needsReview && <span className={styles.reviewTag}>Review</span>}</td>
                  <td><span className={styles.sentiment}>{mention.sentiment}</span></td><td>{mention.confidence}%</td>
                </tr>
              ))}</tbody>
            </table>
          </div>

          <div className={styles.summaryGrid}>
            <article><span>WHAT FANNY FOUND</span><p>Digital experience and community mentions are favorable. The strongest negative signal concerns branch support. Two ambiguous items were withheld for human judgment.</p></article>
            <article><span>WHAT FANNY DID NOT DO</span><p>No source records were changed. No report was sent. No customer credentials or external systems were accessed.</p></article>
            <article><span>RECOMMENDED NEXT STEP</span><p>Review the fraud-related question and competitive comparison, then approve the report draft for team delivery.</p></article>
          </div>

          <div className={styles.approvalBar}>
            <div><strong>{approved ? 'Approved for this demo' : 'Human review required'}</strong><span>{approved ? 'The approval receipt has been recorded in this session.' : 'Fanny stops here until a person decides.'}</span></div>
            <div className={styles.heroActions}>
              <button className={styles.secondary} onClick={() => {setApproved(false); speak('The report was returned for correction.');}}>Request correction</button>
              <button className={styles.primary} onClick={() => {setApproved(true); speak('Approval recorded. In production, this would release only the specifically authorized next step.');}}>Approve demo report</button>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
