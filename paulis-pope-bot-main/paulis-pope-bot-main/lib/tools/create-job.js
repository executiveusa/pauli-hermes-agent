import { v4 as uuidv4 } from 'uuid';
import { githubApi } from './github.js';

/**
 * Create a new job branch with updated job.md
 * @param {string} jobDescription - The job description to write to job.md
 * @returns {Promise<{job_id: string, branch: string}>} - Job ID and branch name
 */
async function createJob(jobDescription) {
  const { GH_OWNER, GH_REPO } = process.env;
  const jobId = uuidv4();
  const branch = `job/${jobId}`;

  // 1. Get main branch SHA
  const mainRef = await githubApi(`/repos/${GH_OWNER}/${GH_REPO}/git/ref/heads/main`);
  const mainSha = mainRef.object.sha;

  // 2. Create new branch
  await githubApi(`/repos/${GH_OWNER}/${GH_REPO}/git/refs`, {
    method: 'POST',
    body: JSON.stringify({
      ref: `refs/heads/${branch}`,
      sha: mainSha,
    }),
  });

  // 3. Create logs/${jobId}/job.md with job content
  await githubApi(`/repos/${GH_OWNER}/${GH_REPO}/contents/logs/${jobId}/job.md`, {
    method: 'PUT',
    body: JSON.stringify({
      message: `job: ${jobId}`,
      content: Buffer.from(jobDescription).toString('base64'),
      branch: branch,
    }),
  });

  return { job_id: jobId, branch };
}

export { createJob };
