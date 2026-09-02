from typing import Dict, List

from openai import OpenAI

from src.config import Config


class Generator:
    def __init__(self):
        self.client = OpenAI(
            api_key=Config.OPENAI_API_KEY,
            base_url=Config.OPENAI_BASE_URL,
        )
        self.model = Config.LLM_MODEL

    def generate(self, question: str, context_chunks: List[Dict[str, any]]) -> str:
        """Generate answer using RAG prompt."""
        if not context_chunks:
            return "No relevant documents found. Please upload a document first."

        context_text = "\n\n".join(
            [
                f"[Document {i + 1}]\n{chunk['text']}"
                for i, chunk in enumerate(context_chunks)
            ]
        )

        prompt = f"""You are a helpful assistant answering questions based on provided documents.

Context from documents:
{context_text}

Question: {question}

Instructions:
- Answer based ONLY on the context provided above
- If the answer is not in the context, say "I couldn't find this information in the uploaded documents."
- Be concise and helpful
- Cite which document(s) you're using when relevant

Answer:"""

        response = self.client.chat.completions.create(
            model=self.model,
            messages=[
                {
                    "role": "system",
                    "content": "You are a helpful assistant that answers questions based on provided documents.",
                },
                {"role": "user", "content": prompt},
            ],
            temperature=0.3,
            max_tokens=1000,
        )

        return response.choices[0].message.content
