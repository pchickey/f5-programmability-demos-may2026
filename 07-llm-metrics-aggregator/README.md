# 07: LLM Metrics Aggregator

This example is really about demonstrating the usefulness of the key-value
store. If you are an iRules programmer, you already know all about the session
table. F5's Wasm Programmability will let you access that same session table
on BIG-IP, and the analogous concept in NGINX, njs's shared dictionary.

For this example, we are going to make a request to a real LLM, running
locally on the ubuntu host in [ollama] at `10.1.1.4:11434`.

[ollama]: https://ollama.com/

Make your request to the LLM using the post body:

```sh
$ curl 10.1.1.4:8000 -d "describe NGINX in 10 words or less"
```

Telling this particular LLM "N words or less" is a useful to make sure it
keeps processing time down, otherwise it takes a long time and can be pretty
verbose.

The LLM is stateless, so we use the key-value store to add on some state. This
state could be especially useful when making routing decisions, if you'd like
to build a bigger application which could, for example, route to a cheaper
model once a cost threshold has been reached on an expensive model.

The response you get back from this program is the entire JSON object returned
by the LLM, plus two more fields: `total_prompt_eval_count` returns a running
sum of the `prompt_eval_count` field reported by the LLM, and
`total_eval_count` returns a running sum of the `eval_count` field reported by
the LLM.
