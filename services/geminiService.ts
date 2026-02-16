
import { GoogleGenAI } from "@google/genai";
import { ScannedHost } from "../types";

const ai = new GoogleGenAI({ apiKey: process.env.API_KEY });

export async function analyzeSecurity(host: ScannedHost): Promise<string> {
  try {
    const portsStr = host.ports.map(p => `${p.port} (${p.service})`).join(', ');
    const prompt = `
      As a cybersecurity expert, analyze this local network host:
      IP: ${host.ip}
      Open Ports: ${portsStr}
      
      Provide a brief (2-3 sentence) security assessment. Mention potential risks for the specific services found.
      Keep it professional and concise.
    `;

    const response = await ai.models.generateContent({
      model: 'gemini-3-flash-preview',
      contents: prompt,
    });

    return response.text || "No security analysis available.";
  } catch (error) {
    console.error("AI Analysis failed:", error);
    return "Analysis unavailable at this time.";
  }
}
