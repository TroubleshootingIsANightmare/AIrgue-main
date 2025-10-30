"use client";
import { Dispatch, FormEvent, SetStateAction, useState } from "react";
import send from "./send";
import get_aires from "./get_aires";

const id = "" + Math.random();




export function MessageArea() {



    const [msgs, setMsgs] = useState<string[]>([]);

    return (
        <div>
            <MessageLog msgs={msgs} />
            <ChatBox msgs={msgs} setMsgs={setMsgs} />
        </div>
    );
}

type MsgsRW = { msgs: string[]; setMsgs: Dispatch<SetStateAction<string[]>> };

export function ChatBox({ msgs, setMsgs }: MsgsRW) {
    const [vin, setVin] = useState("");

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault(); // prevent newline
            // Manually call onSubmit
            onSubmit(
                e as unknown as FormEvent<HTMLFormElement>,
                vin,
                setVin,
                msgs,
                setMsgs
            );
        }
    };
    
    return (
        <div id="cb">
            <form
                id="chatbox"
                onSubmit={(e) => onSubmit(e, vin, setVin, msgs, setMsgs)}
            >
                <textarea
                    id="cbin"
                    value={vin}
                    onChange={(e) => setVin(e.target.value)}
                    onKeyDown={handleKeyDown}
                ></textarea>
                <input type="submit" />
            </form>
        </div>
    );
}

export function MessageLog({ msgs }: { msgs: string[] }) {
    return (
        <div className="message_box">
            {msgs.map((m, i) => {
                // Determine class based on index: even = user, odd = AI
                const senderClass = i % 2 === 0 ? "user" : "ai";

                return (
                    <div key={i} className={`message ${senderClass}`}>
                        {m}
                    </div>
                );
            })}
        </div>
    );
}

// --- FIXED onSubmit ---

async function onSubmit(
    event: FormEvent<HTMLFormElement>,
    vin: string,
    setVin: Dispatch<SetStateAction<string>>,
    msgs: string[],
    setMsgs: Dispatch<SetStateAction<string[]>>
) {
    event.preventDefault();

    if (!vin.trim()) return;

    setMsgs((prev) => [...prev, vin]);
    send(vin, id);
    setVin("");

    // add empty message to be filled during streaming
    setMsgs((prev) => [...prev, ""]);

    let streaming = true;
    while (streaming) {
        const word = await get_aires(id);

        if (!word) {
            streaming = false;
            break;
        }

        setMsgs((prev) => {
            const copy = [...prev];
            copy[copy.length - 1] = copy[copy.length - 1] + word + " ";
            return copy;
        });

        await new Promise((resolve) => setTimeout(resolve, 20));
    }
}
