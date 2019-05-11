package com.github.apohrebniak.hoverui.ui;

import com.github.apohrebniak.hoverui.domain.MemberEntity;
import com.vaadin.flow.component.button.Button;
import com.vaadin.flow.component.html.Div;
import com.vaadin.flow.component.html.Label;
import com.vaadin.flow.component.icon.Icon;
import com.vaadin.flow.component.icon.VaadinIcon;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.router.RouterLink;

public class MemberListItem extends VerticalLayout {
  private static final String BUTTON_TEXT = "Visit";

  public static MemberListItem from(MemberEntity entity) {
    MemberListItem item = new MemberListItem();
    item.getStyle().set("background-color", "lavender");
    item.getStyle().set("border-radius", "8px");
    item.foo(String.format("Hover Id: %s", entity.getId()));
    item.foo(String.format("Http address: %s:%d", entity.getHttpAddress().getIp(), entity.getHttpAddress().getPort()));
    item.foo(String.format("Hover node address: %s:%d", entity.getHoverNode().getAddr().getIp(), entity.getHoverNode().getAddr().getPort()));
    item.add(buildLink(entity));
    return item;
  }

  private static RouterLink buildLink(MemberEntity entity) {
    MemberEntity.Address http = entity.getHttpAddress();
    Button button = new Button(BUTTON_TEXT, new Icon(VaadinIcon.EXTERNAL_LINK));
    RouterLink routerLink = new RouterLink("", NodeView.class, http.getIp() + "_" + http.getPort());
    routerLink.getElement().appendChild(button.getElement());

    return routerLink;
  }

  private void foo(String text) {
    Label label = new Label(text);
    label.getStyle().set("font-size", "20px");
    label.getStyle().set("font-family", "monospace");
    add(new Div(label));
  }
}
