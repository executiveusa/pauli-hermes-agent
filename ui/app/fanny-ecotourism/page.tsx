'use client';

import { useMemo, useState } from 'react';
import styles from '../fanny/fanny.module.css';

const steps = ['Listen', 'Clarify', 'Challenge', 'Structure', 'Draft', 'Review'];

export default function FannyEcotourismPage() {
  const [stage, setStage] = useState(-1);
  const [idea, setIdea] = useState('I run a small nature and food experience and want more bilingual bookings without wasting money on random posts.');
  const [approved, setApproved] = useState(false);
  const [speaking, setSpeaking] = useState(false);

  const brief = useMemo(() => ({
    outcome: 'Generate qualified bilingual inquiries for one bookable experience.',
    customer: 'Spanish- and English-speaking visitors who value local culture, food, and nature.',
    assumption: 'The audience will respond to the combined nature-and-food angle.',
    test: 'Seven Postiz drafts across two channels, reviewed before scheduling.',
    metric: 'Qualified inquiries and completed bookings—not likes alone.',
  }), []);

  function speak(text: string) {
    if (!('speechSynthesis' in window)) return;
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = 'es-MX';
    utterance.rate = 0.94;
    utterance.onstart = () => setSpeaking(true);
    utterance.onend = () => setSpeaking(false);
    window.speechSynthesis.speak(utterance);
  }

  function runDemo() {
    setApproved(false);
    setStage(0);
    speak('Hola. Soy Fanny. Voy a escuchar tu idea, separar lo que sabemos de lo que asumimos, y proponerte una prueba pequeña antes de que gastes dinero.');
    steps.forEach((_, index) => window.setTimeout(() => setStage(index), index * 720));
    window.setTimeout(() => setStage(steps.length), steps.length * 720);
  }

  return (
    <main className={styles.page}>
      <section className={styles.hero}>
        <div>
          <p className={styles.eyebrow}>PUERTO VALLARTA · CDMX · WASHINGTON</p>
          <h1>Fanny<br/>Ecotourism.</h1>
          <p className={styles.lede}>Speak naturally in Spanish or English. Fanny turns the idea into a testable offer, challenges weak assumptions, and prepares reviewed Postiz content.</p>
          <div className={styles.actions}>
            <button className={styles.primary} onClick={runDemo}>Run voice-first demo</button>
            <button className={styles.secondary} onClick={() => speak('No necesito que escribas un prompt perfecto. Cuéntame la idea como se la contarías a una socia. Yo la voy a organizar contigo.')}>Hear Fanny</button>
          </div>
        </div>
        <aside className={styles.avatarCard}>
          <div className={`${styles.avatar} ${speaking ? styles.active : ''}`}><span>F</span><i /></div>
          <strong>Fanny</strong><p>{speaking ? 'Speaking · Español' : stage < 0 ? 'Ready to listen' : stage >= steps.length ? 'Campaign brief ready' : `Working · ${steps[stage]}`}</p>
          <small>Voice prototype · ElevenLabs adapter planned</small>
        </aside>
      </section>

      <section className={styles.workspace}>
        <article className={styles.panel}>
          <div className={styles.heading}><div><span>01 · YOUR IDEA</span><h2>Talk first. Structure second.</h2></div><em>Voice-first</em></div>
          <textarea value={idea} onChange={(event) => setIdea(event.target.value)} aria-label="Business idea" />
          <p className={styles.guardrail}>This browser prototype uses sample processing. Live voice, customer memory, and Postiz require tenant-specific credentials and approval.</p>
        </article>
        <article className={styles.panel}>
          <div className={styles.heading}><div><span>02 · ICM + GRILL ME</span><h2>She does not just agree.</h2></div><em>Anti-sycophancy</em></div>
          <div className={styles.pipeline}>{steps.map((name, index) => <div key={name} className={`${styles.stage} ${stage === index ? styles.current : ''} ${stage > index ? styles.done : ''}`}><b>{String(index + 1).padStart(2, '0')}</b><span>{name}</span><small>{stage > index ? 'Complete' : stage === index ? 'Working' : 'Waiting'}</small></div>)}</div>
          <p className={styles.guardrail}><strong>Challenge:</strong> The combined offer sounds promising, but demand is unproven. Test one audience and one offer before expanding the budget.</p>
        </article>
      </section>

      <section className={styles.results}>
        <div className={styles.resultHeading}><div><span>03 · ENHANCED CAMPAIGN BRIEF</span><h2>Evidence before spending.</h2></div><em>Postiz default: drafts only</em></div>
        <div className={styles.summaries}>
          <article><span>OUTCOME</span><p>{brief.outcome}</p></article>
          <article><span>CUSTOMER</span><p>{brief.customer}</p></article>
          <article><span>UNPROVEN</span><p>{brief.assumption}</p></article>
          <article><span>SMALLEST TEST</span><p>{brief.test}</p></article>
          <article><span>METRIC</span><p>{brief.metric}</p></article>
          <article><span>FANNY'S PUSHBACK</span><p>Do not scale until inquiries or bookings demonstrate useful demand.</p></article>
        </div>
        <div className={styles.approval}>
          <div><strong>{approved ? 'Approved for draft creation' : 'Human review required'}</strong><span>No scheduling or publishing occurs from this prototype.</span></div>
          <div className={styles.actions}><button className={styles.secondary} onClick={() => setApproved(false)}>Revise idea</button><button className={styles.primary} onClick={() => setApproved(true)}>Approve draft test</button></div>
        </div>
      </section>
    </main>
  );
}
