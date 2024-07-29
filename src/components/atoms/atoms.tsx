import { useEffect, useRef, MouseEventHandler } from 'react';
import { useNavigate } from "react-router-dom";


import IconStyles from "./styles/IconStyles.module.css";
import ContextMenuStyles from "./styles/ContextMenuStyles.module.css";
import LoaderStyles from "./styles/LoaderStyles.module.css";

interface IconProps {
    routerPath: string,
    iconPath: string,
    isSelected?: boolean,
}
export function Icon(props: IconProps) {
    const navigate = useNavigate();
    const onClick = () => {
        navigate(props.routerPath);
    }

    var styles = {backgroundColor: "transparent"};
    if (props.isSelected != undefined && props.isSelected) {
        styles.backgroundColor = "rgb(38, 38, 38)";
    }

    return (<>
        <div className={IconStyles.container} style={styles}>
            <img 
                src={props.iconPath} 
                className={IconStyles.icon}
                onClick={onClick}
            />
        </div>

    </>)
}

interface ContextMenuProps {
    options: Array<{label: string, func: MouseEventHandler<HTMLParagraphElement>}>,
    isOpen?: boolean,
    onClose: () => void,
}

export function ContextMenu({options, isOpen = false, onClose}: ContextMenuProps) {
    const menuRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                onClose();
            }
        };

        if (isOpen) {
            document.addEventListener("mousedown", handleClickOutside);
        }

        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    return (
        <div className={ContextMenuStyles.container} ref={menuRef}>
            <div className={ContextMenuStyles.settings_popup_menu}>
                {options.map(elem => (
                    <p 
                        key={elem.label}
                        className={ContextMenuStyles.settings_popup_menu_section}
                        onClick={(e) => {
                            elem.func(e);
                            onClose();
                        }}
                    >
                        {elem.label}
                    </p>
                ))}
            </div>
        </div>
    );
}

export function Loader() {
    return (<>
        <div className={LoaderStyles.dot_spinner}>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
            <div className={LoaderStyles.dot_spinner__dot}></div>
        </div>
    </>)
}