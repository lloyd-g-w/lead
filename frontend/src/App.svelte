<script lang="ts">
  const ws = new WebSocket("ws://127.0.0.1:7050");
  ws.onmessage = (e) => console.log("->", e.data);
  let input: string = "";
</script>

<main>
  <h1>Lead</h1>
  <button onclick={() => ws.send(input)}>Send msg</button>
  <input bind:value={input} />
</main>

<style>
</style>
