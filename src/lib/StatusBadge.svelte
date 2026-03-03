<script lang="ts">
  import type { PullRequest } from "./types";

  interface Props {
    pr: PullRequest;
  }
  let { pr }: Props = $props();

  let color = $derived.by(() => {
    if (pr.state === "merged") return "bg-accent";
    if (pr.is_draft) return "bg-gray-500";
    if (pr.merge_queue_info) return "bg-yellow-500";
    switch (pr.check_status) {
      case "success": return "bg-green-500";
      case "failure":
      case "error": return "bg-red-500";
      case "pending": return "bg-yellow-500";
      default: return "bg-gray-500";
    }
  });
</script>

<span class="w-2 h-2 rounded-full shrink-0 {color}"></span>
