<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import LifeDisplay from "./LifeDisplay.svelte";
  import ManaDisplay from "./ManaDisplay.svelte";

  let name = $state("");
  let health = $state({ hp: 1, max_hp: 1, unreserved_hp: 1, es: 1, max_es: 1 });
  let mana = $state({ mana: 1, max_mana: 1, unreserved_mana: 1 });

  async function greet(event: Event) {
    event.preventDefault();
    health = await invoke("greet", { name });
  }
</script>

<div class="container">
  <h1>poe-hacks</h1>
  <form class="row" onsubmit={greet}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{JSON.stringify(health)}</p>
  <p>{(health.max_hp - health.hp) / health.max_hp}</p>

  <div class="resource-displays">
    <LifeDisplay {health} />
    <ManaDisplay {mana} />
  </div>
</div>

<style>
  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  .row {
    display: flex;
    justify-content: center;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }

  .resource-displays {
    display: flex;
    justify-content: space-between;
    height: 10em;
  }
</style>
