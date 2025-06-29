<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import LifeDisplay from "./LifeDisplay.svelte";
  import ManaDisplay from "./ManaDisplay.svelte";

  let name = $state("");
  let health = $state({ hp: 1, maxHp: 1, unreservedHp: 1, es: 1, maxEs: 1 });
  let mana = $state({ mana: 1, maxMana: 1, unreservedMana: 1 });

  async function greet(event: Event) {
    event.preventDefault();
    health = await invoke("greet", { name });
  }
</script>

<div class="status">
  <h1>poe-hacks</h1>
</div>
<div class="resource-displays">
  <LifeDisplay {health} />
  <ManaDisplay {mana} />
</div>

<style>
  h1 {
    text-align: center;
  }

  .status {
    flex-grow: 1;
  }

  .resource-displays {
    display: flex;
    justify-content: space-between;
    height: 10em;
  }
</style>
