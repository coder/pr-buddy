<script lang="ts">
  import type { PullRequest } from "./types";

  interface Props {
    pr: PullRequest;
  }
  let { pr }: Props = $props();

  let color = $derived.by(() => {
    if (pr.state === "merged") return "bg-accent";
    if (pr.is_draft) return "bg-neutral-400";
    if (pr.merge_queue_info) return "bg-amber-500";
    switch (pr.check_status) {
      case "success": return "bg-emerald-500";
      case "failure":
      case "error": return "bg-red-500";
      case "pending": return "bg-amber-500";
      default: return "bg-neutral-400";
    }
  });
</script>

<span class="w-2 h-2 rounded-full shrink-0 {color}"></span>
