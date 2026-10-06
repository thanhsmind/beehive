import { pathToFileURL } from "node:url";

const args = process.argv.slice(1);
const shift = args[0] === "--" || args[0] === "[eval]" ? 1 : 0;
const [cliDir, agentId, text] = args.slice(shift);

try {
  if (!cliDir || !agentId || !text) {
    throw new Error("missing required arguments: cliDir, agentId, text");
  }
  const { connectToDaemon } = await import(pathToFileURL(cliDir + "/dist/utils/client.js").href);
  const { selectDaemonTarget } = await import(pathToFileURL(cliDir + "/dist/utils/daemon-target.js").href);
  const client = await connectToDaemon({ target: selectDaemonTarget({}, process.env, false) });
  try {
    await client.sendAgentMessage(agentId, text, { activeTurnBehavior: "steer" });
  } finally {
    if (client && typeof client.close === "function") {
      await client.close();
    }
  }
  console.log(JSON.stringify({ ok: true }));
} catch (err) {
  const msg = err && err.message ? err.message : String(err);
  console.log(JSON.stringify({ ok: false, error: msg }));
  process.exit(1);
}
