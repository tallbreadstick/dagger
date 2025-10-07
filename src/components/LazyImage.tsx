import { createEffect, createSignal, onCleanup } from "solid-js";

export function LazyImage(props: { src: string; alt: string; class?: string }) {
    let imgRef: HTMLImageElement | undefined;
    const [visible, setVisible] = createSignal(false);

    createEffect(() => {
        if (!imgRef) return;
        const observer = new IntersectionObserver(
            (entries) => {
                entries.forEach((entry) => {
                    if (entry.isIntersecting) {
                        setVisible(true);
                        observer.unobserve(entry.target);
                    }
                });
            },
            { rootMargin: "100px" } // small preload margin
        );
        observer.observe(imgRef);
        onCleanup(() => observer.disconnect());
    });

    return (
        <img
            ref={imgRef}
            src={visible() ? props.src : undefined}
            alt={props.alt}
            class={props.class}
            loading="lazy"
        />
    );
}
